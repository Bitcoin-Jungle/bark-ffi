//! Nostr Wallet Connect (NIP-47): lets an app drive the wallet through a nostr
//! relay, with a connection URI the wallet hands it.
//!
//! The relay services and the request handling are upstream's
//! [`bark::nwc::NwcManager`], which the wallet starts and stops with its
//! daemon. What is here is the store the connections and relays live in, and
//! keeping a new connection unserved while the daemon is stopped, which
//! upstream does not.
//!
//! Upstream persists through a synchronous [`ConnectionPersister`], which an
//! IndexedDB store cannot be. So every change is written ahead here, through
//! the async [`NwcStore`], and only then applied to the manager, whose own
//! persister does nothing. Changes are serialized, so what gets written is
//! exactly what the manager holds next, and a write that fails changes
//! nothing.

use std::collections::HashSet;
use std::sync::Arc;

use anyhow::Context;
use log::{info, warn};

use bark::nwc::{Connection, ConnectionPersister, ConnectionStore, NwcManager, RelayUrl};

use crate::error::Error;
use crate::types::{NwcConnection, NwcConnectionUri, NwcMethod};

/// Where a wallet's NWC connections and relays are kept.
///
/// The store holds every connection's service secret, so keep it as private
/// as the wallet's own data.
#[cfg_attr(feature = "wasm-web", async_trait::async_trait(?Send))]
#[cfg_attr(not(feature = "wasm-web"), async_trait::async_trait)]
pub(crate) trait NwcStore: Send + Sync + 'static {
    /// The stored connections and relays, or an empty store if none was saved
    /// yet. Must fail rather than return an empty store when what is stored
    /// cannot be read: the next save would overwrite it, revoking every
    /// connection.
    async fn load(&self) -> anyhow::Result<ConnectionStore>;

    /// Replace what is stored with `store`.
    async fn save(&self, store: &ConnectionStore) -> anyhow::Result<()>;
}

/// The manager's persister. Nothing is left to do by the time the manager
/// calls it: [`Nwc`] already wrote the change.
struct WrittenAhead;

impl ConnectionPersister for WrittenAhead {
    fn save(&self, _: &ConnectionStore) -> anyhow::Result<()> {
        Ok(())
    }
}

/// A wallet's NWC: its connections, the relays they are served on, and the
/// store both are kept in.
pub(crate) struct Nwc {
    manager: Arc<NwcManager>,
    store: Arc<dyn NwcStore>,
    /// Held across each change, from reading the current state to applying
    /// the changed one, so two changes cannot both build on the same state
    /// and lose one.
    change: tokio::sync::Mutex<()>,
    /// Held across starting, stopping, adding and relay changes, so none of
    /// them sees another half done.
    gate: std::sync::Mutex<Gate>,
}

/// Whether connections are served.
///
/// Upstream starts a connection's service when it is added, daemon or not,
/// and a relay change made while the daemon is stopped would not reach that
/// service. So while connections are not served a new one only goes into the
/// store, and the daemon serves it with the rest when it starts.
#[derive(Debug, Default)]
struct Gate {
    /// Whether the daemon was last asked to run.
    wanted: bool,
    /// How many stops are still waiting for the daemon's tasks to finish.
    ///
    /// A start meanwhile is left to the last of them. Started at once,
    /// upstream would take the daemon still winding down as running, and its
    /// stop could race a relay change into serving with no daemon.
    stopping: usize,
}

impl Gate {
    /// Asked to start. Returns whether to start the daemon now.
    fn start(&mut self) -> bool {
        self.wanted = true;
        self.stopping == 0
    }

    /// Asked to stop, without waiting.
    fn stop(&mut self) {
        self.wanted = false;
    }

    /// Asked to stop and wait.
    fn begin_wait(&mut self) {
        self.wanted = false;
        self.stopping += 1;
    }

    /// A wait is done. Returns whether to start the daemon a start asked for
    /// during it.
    fn end_wait(&mut self) -> bool {
        self.stopping -= 1;
        self.serving()
    }

    fn serving(&self) -> bool {
        self.wanted && self.stopping == 0
    }
}

/// Register NWC on `wallet` with what `store` holds. Call before the daemon
/// starts: serving starts with it.
pub(crate) async fn register(
    wallet: &bark::Wallet,
    store: Arc<dyn NwcStore>,
) -> anyhow::Result<Arc<Nwc>> {
    #[cfg(not(target_arch = "wasm32"))]
    install_crypto_provider();

    let connections = store.load().await.context("failed to load NWC connections")?;
    let manager = wallet
        .init_nwc(connections, Arc::new(WrittenAhead))
        .context("failed to register NWC")?;

    {
        let connections = manager.connections().read();
        info!(
            "[NWC] {} connection(s) on {} relay(s)",
            connections.iter().count(),
            connections.relays().len(),
        );
    }

    Ok(Arc::new(Nwc {
        manager,
        store,
        change: tokio::sync::Mutex::new(()),
        gate: std::sync::Mutex::new(Gate::default()),
    }))
}

/// Pick the rustls crypto provider relays are dialled with.
///
/// rustls panics on the first handshake when it cannot tell which provider
/// to use, which happens when a build unifies a second provider feature
/// (aws-lc-rs) into the graph. Installing ring keeps the choice the same
/// however bark-ffi is built.
#[cfg(not(target_arch = "wasm32"))]
fn install_crypto_provider() {
    // An error means a provider is installed already, which works as well.
    let _ = rustls::crypto::ring::default_provider().install_default();
}

impl Nwc {
    /// Start `wallet`'s daemon, and with it serving every connection. While
    /// a stop waits, the daemon starts once that is done.
    pub(crate) fn start_daemon(&self, wallet: &bark::Wallet) -> anyhow::Result<()> {
        let mut gate = self.gate();
        if gate.start() {
            if let Err(e) = wallet.start_daemon() {
                gate.stop();
                return Err(e);
            }
        }
        Ok(())
    }

    /// Stop `wallet`'s daemon, and with it serving every connection.
    pub(crate) fn stop_daemon(&self, wallet: &bark::Wallet) {
        let mut gate = self.gate();
        gate.stop();
        // Stops the services too.
        wallet.stop_daemon();
    }

    /// Stop `wallet`'s daemon, and with it serving every connection, and
    /// wait until its tasks have finished.
    pub(crate) async fn stop_daemon_wait(&self, wallet: &bark::Wallet) -> anyhow::Result<()> {
        let mut waiting = Waiting::begin(self, wallet);
        // The gate cannot be held across this. What could change meanwhile
        // starts nothing: starts are left to `waiting`.
        let stopped = wallet.stop_daemon_wait().await;
        waiting.end()?;
        stopped
    }

    fn gate(&self) -> std::sync::MutexGuard<'_, Gate> {
        // Only flags behind it, which the next start or stop sets again.
        self.gate.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Create a connection granting `methods`, served while the daemon runs.
    ///
    /// Returns the URI to hand to the app. It carries the app's secret key,
    /// which the wallet does not keep, so it cannot be shown again.
    pub(crate) async fn create_connection(
        &self,
        name: String,
        methods: Vec<NwcMethod>,
    ) -> Result<NwcConnectionUri, Error> {
        if methods.is_empty() {
            return Err("an NWC connection needs at least one method".into());
        }
        let methods = methods.iter().map(|m| m.as_str().to_owned()).collect::<HashSet<_>>();

        let _change = self.change.lock().await;
        let relays = self.manager.relays();
        // The URI tells the app where to find the wallet, so a connection
        // made without relays could never be reached.
        if relays.is_empty() {
            return Err("no NWC relays are configured: add one first".into());
        }

        let (connection, uri) = Connection::generate(name, relays, methods, chrono::Utc::now());
        let client_pubkey = connection.client_pubkey.clone();

        let mut next = self.current();
        next.insert(connection.clone());
        self.store.save(&next).await.context("failed to save the NWC connection")?;
        let gate = self.gate();
        if gate.serving() {
            self.manager.add(connection)?;
        } else {
            // `add` would serve it now. The daemon serves it once it starts.
            self.manager.connections().write().insert(connection);
        }
        drop(gate);

        Ok(NwcConnectionUri { uri: uri.to_string(), client_pubkey })
    }

    /// Every connection, oldest first.
    pub(crate) fn connections(&self) -> Vec<NwcConnection> {
        let mut connections = self.manager.connections().read()
            .iter()
            .map(|c| (c.created_at, connection_info(c)))
            .collect::<Vec<_>>();
        connections.sort_by(|(a_time, a), (b_time, b)| {
            a_time.cmp(b_time).then_with(|| a.client_pubkey.cmp(&b.client_pubkey))
        });
        connections.into_iter().map(|(_, c)| c).collect()
    }

    /// Revoke the connection of the app with `client_pubkey` and stop
    /// serving it. Returns whether there was such a connection.
    pub(crate) async fn revoke_connection(&self, client_pubkey: &str) -> Result<bool, Error> {
        let client_pubkey = client_pubkey.trim().to_ascii_lowercase();

        let _change = self.change.lock().await;
        let mut next = self.current();
        if next.remove(&client_pubkey).is_some() {
            self.store.save(&next).await
                .context("failed to save the NWC connection removal")?;
        }
        // Stops a service for this key even when the store had no
        // connection for it.
        Ok(self.manager.revoke(&client_pubkey)?)
    }

    /// The relays connections are served on.
    pub(crate) fn relays(&self) -> Vec<String> {
        relay_strings(&self.manager.relays())
    }

    /// Serve every connection on `relay` too. A no-op if it is served on
    /// already. Returns the relays after the change.
    pub(crate) async fn add_relay(&self, relay: &str) -> Result<Vec<String>, Error> {
        let relay = parse_relay(relay)?;

        let _change = self.change.lock().await;
        let mut relays = self.manager.relays();
        if !relays.contains(&relay) {
            relays.push(relay);
            self.set_relays(relays.clone()).await?;
        }
        Ok(relay_strings(&relays))
    }

    /// Stop serving connections on `relay`. Returns the relays after the
    /// change.
    pub(crate) async fn remove_relay(&self, relay: &str) -> Result<Vec<String>, Error> {
        let relay = parse_relay(relay)?;

        let _change = self.change.lock().await;
        let mut relays = self.manager.relays();
        let before = relays.len();
        relays.retain(|r| r != &relay);
        if relays.len() == before {
            return Err(format!("no such NWC relay: {}", relay).into());
        }
        self.set_relays(relays.clone()).await?;
        Ok(relay_strings(&relays))
    }

    /// Write `relays` ahead and apply them. Call with `change` held.
    async fn set_relays(&self, relays: Vec<RelayUrl>) -> Result<(), Error> {
        let mut next = self.current();
        next.set_relays(relays.clone());
        self.store.save(&next).await.context("failed to save the NWC relays")?;
        // Upstream restarts the services only while it serves. Held so a stop
        // cannot land between that check and the restart.
        let gate = self.gate();
        self.manager.set_relays(relays)?;
        drop(gate);
        Ok(())
    }

    /// A copy of the manager's store, to change and write ahead. The guard
    /// is dropped on return, so it never lives across an await.
    fn current(&self) -> ConnectionStore {
        self.manager.connections().read().clone()
    }

    #[cfg(test)]
    pub(crate) fn serving(&self) -> bool {
        self.gate().serving()
    }
}

/// A stop waiting for the daemon's tasks to finish. A start asked for
/// meanwhile is made when it ends, or when it is dropped if the wait is
/// abandoned, so it cannot be lost.
struct Waiting<'a> {
    nwc: &'a Nwc,
    wallet: &'a bark::Wallet,
    ended: bool,
}

impl<'a> Waiting<'a> {
    fn begin(nwc: &'a Nwc, wallet: &'a bark::Wallet) -> Self {
        let mut gate = nwc.gate();
        gate.begin_wait();
        // Upstream's stop stops them too, but outside the gate, where a
        // relay change could restart them.
        nwc.manager.stop_all();
        drop(gate);
        Self { nwc, wallet, ended: false }
    }

    /// Start the daemon if a start was asked for during the wait.
    fn end(&mut self) -> anyhow::Result<()> {
        if std::mem::replace(&mut self.ended, true) {
            return Ok(());
        }
        let mut gate = self.nwc.gate();
        if gate.end_wait() {
            if let Err(e) = self.wallet.start_daemon() {
                gate.stop();
                return Err(e.context("failed to start the daemon a start asked for"));
            }
        }
        Ok(())
    }
}

impl Drop for Waiting<'_> {
    fn drop(&mut self) {
        // Does something only when the wait was abandoned.
        if let Err(e) = self.end() {
            warn!("[NWC] {:#}", e);
        }
    }
}

fn connection_info(c: &Connection) -> NwcConnection {
    NwcConnection {
        name: c.name.clone(),
        client_pubkey: c.client_pubkey.clone(),
        // In a fixed order. A method this version does not know is left out.
        methods: NwcMethod::ALL
            .into_iter()
            .filter(|m| c.allowed_methods.contains(m.as_str()))
            .collect(),
        created_at: c.created_at.to_rfc3339(),
    }
}

fn parse_relay(relay: &str) -> Result<RelayUrl, Error> {
    let mut relays = bark::nwc::parse_relays(&[relay.trim().to_owned()])?;
    Ok(relays.pop().expect("one relay in, one out"))
}

fn relay_strings(relays: &[RelayUrl]) -> Vec<String> {
    relays.iter().map(|r| r.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A method upstream serves that [`NwcMethod`] does not name could
    /// never be granted, and listing would drop it.
    #[test]
    fn every_served_method_has_an_nwc_method() {
        let served = bark::nwc::dispatch::supported_methods()
            .iter()
            .map(|m| m.to_string())
            .collect::<Vec<_>>();
        for name in &served {
            assert!(NwcMethod::from_name(name).is_some(), "no NwcMethod for {}", name);
        }
        for method in NwcMethod::ALL {
            assert!(served.iter().any(|s| s == method.as_str()), "{:?} not served", method);
        }
    }

    #[test]
    fn method_names_match_their_serde_names() {
        for method in NwcMethod::ALL {
            let json = serde_json::to_value(method).unwrap();
            assert_eq!(json, serde_json::json!(method.as_str()));
        }
    }

    /// Started at once, a start while a stop waits let upstream's stop race
    /// a relay change into serving with no daemon.
    #[test]
    fn a_start_while_a_stop_waits_is_left_to_it() {
        let mut gate = Gate::default();
        assert!(gate.start());
        gate.begin_wait();
        assert!(!gate.start(), "started while a stop waits");
        assert!(!gate.serving(), "a new connection would be served mid-stop");
        assert!(gate.end_wait(), "the start asked for was dropped");
        assert!(gate.serving());
    }

    #[test]
    fn a_stop_after_a_left_start_cancels_it() {
        let mut gate = Gate::default();
        gate.begin_wait();
        assert!(!gate.start());
        gate.stop();
        assert!(!gate.end_wait());
        assert!(!gate.serving());
    }

    #[test]
    fn only_the_last_of_overlapping_waits_starts() {
        let mut gate = Gate::default();
        gate.begin_wait();
        gate.begin_wait();
        assert!(!gate.start());
        assert!(!gate.end_wait());
        assert!(!gate.serving());
        assert!(gate.end_wait());
        assert!(gate.serving());
    }

    #[test]
    fn a_relay_must_be_a_websocket_url() {
        assert!(parse_relay("https://relay.example.com").is_err());
        assert!(parse_relay("relay.example.com").is_err());
        let relay = parse_relay(" wss://relay.example.com ").unwrap();
        assert_eq!(relay, parse_relay("wss://relay.example.com").unwrap());
    }
}
