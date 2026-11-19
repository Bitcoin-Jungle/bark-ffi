//! Nostr Wallet Connect (NIP-47) over uniffi. See [`crate::core::nwc`].
//!
//! Connections and relays are kept in `nwc.json` in the datadir, the file
//! barkd keeps them in, next to the wallet database rather than in it.

use std::path::{Path, PathBuf};

use anyhow::Context;

use bark::nwc::ConnectionStore;

use crate::core::nwc::NwcStore;
use crate::muniffi::runtime::run_async;
use crate::muniffi::wallet::Wallet;
use crate::{types, Error};

/// Name of the NWC connection store inside the datadir.
pub(crate) const NWC_FILE: &str = "nwc.json";

/// The NWC connection store as a file in the datadir.
///
/// It holds every connection's service secret, so it is written owner-only
/// on unix, and atomically: a torn write would lose every connection.
pub(crate) struct FileNwcStore {
    path: PathBuf,
}

impl FileNwcStore {
    pub(crate) fn new(datadir: &Path) -> Self {
        Self { path: datadir.join(NWC_FILE) }
    }
}

#[async_trait::async_trait]
impl NwcStore for FileNwcStore {
    async fn load(&self) -> anyhow::Result<ConnectionStore> {
        let bytes = match std::fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(ConnectionStore::new());
            },
            Err(e) => {
                return Err(e).with_context(|| format!("failed to read {}", self.path.display()));
            },
        };
        serde_json::from_slice(&bytes)
            .with_context(|| format!("failed to parse {}", self.path.display()))
    }

    async fn save(&self, store: &ConnectionStore) -> anyhow::Result<()> {
        let json = serde_json::to_vec_pretty(store)
            .context("failed to serialize NWC connections")?;
        if cfg!(unix) {
            bark::fs_perms::write_atomic_owner_only(&self.path, &json)
        } else {
            // Upstream's helper writes in place off unix, so a failed write
            // would leave a torn file.
            replace_via_temp(&self.path, &json)
        }
    }
}

/// Replace `path` with `contents` through a temp sibling, so a failed write
/// leaves the old file whole.
fn replace_via_temp(path: &Path, contents: &[u8]) -> anyhow::Result<()> {
    use std::io::Write;

    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    let written = (|| -> anyhow::Result<()> {
        let mut file = std::fs::File::create(&tmp)
            .with_context(|| format!("failed to create {}", tmp.display()))?;
        file.write_all(contents)
            .with_context(|| format!("failed to write {}", tmp.display()))?;
        file.sync_all()
            .with_context(|| format!("failed to flush {}", tmp.display()))?;
        // Closed first: Windows does not rename an open file.
        drop(file);
        std::fs::rename(&tmp, path)
            .with_context(|| format!("failed to replace {}", path.display()))
    })();
    if written.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    written
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replacing_via_temp_leaves_only_the_new_file() {
        let dir = std::env::temp_dir().join(format!("bark-ffi-nwc-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(NWC_FILE);

        replace_via_temp(&path, b"old").unwrap();
        replace_via_temp(&path, b"new").unwrap();

        assert_eq!(std::fs::read(&path).unwrap(), b"new");
        let names = std::fs::read_dir(&dir).unwrap()
            .map(|e| e.unwrap().file_name())
            .collect::<Vec<_>>();
        assert_eq!(names, vec![std::ffi::OsString::from(NWC_FILE)]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl Wallet {
    // ------------------------------------------------------------------------
    // Nostr Wallet Connect
    // ------------------------------------------------------------------------

    /// Create a Nostr Wallet Connect (NIP-47) connection that lets an app
    /// call `methods` on this wallet.
    ///
    /// Needs a relay, see [`Self::nwc_add_relay`]. Returns the URI to hand to
    /// the app. It carries the app's secret key, which the wallet does not
    /// keep: show it once and never log or store it. `PayInvoice` lets the
    /// app spend the whole balance, there is no spending limit.
    ///
    /// Requests are answered while the daemon runs. A connection created
    /// while it is stopped is served once it starts.
    pub async fn nwc_create_connection(
        &self,
        name: String,
        methods: Vec<types::NwcMethod>,
    ) -> Result<types::NwcConnectionUri, Error> {
        let core = self.core().clone();
        run_async(async move { core.nwc()?.create_connection(name, methods).await }).await
    }

    /// Every NWC connection, oldest first.
    pub fn nwc_connections(&self) -> Result<Vec<types::NwcConnection>, Error> {
        Ok(self.core().nwc()?.connections())
    }

    /// Revoke the NWC connection of the app with `client_pubkey` and stop
    /// serving it. Returns whether there was such a connection.
    pub async fn nwc_revoke_connection(&self, client_pubkey: String) -> Result<bool, Error> {
        let core = self.core().clone();
        run_async(async move { core.nwc()?.revoke_connection(&client_pubkey).await }).await
    }

    /// The relays NWC connections are served on.
    pub fn nwc_relays(&self) -> Result<Vec<String>, Error> {
        Ok(self.core().nwc()?.relays())
    }

    /// Serve NWC connections on `relay` too, a `ws://` or `wss://` URL.
    /// Returns the relays after the change.
    ///
    /// A connection's URI names the relays it was created with, so its app
    /// only finds the wallet on relays from that list.
    pub async fn nwc_add_relay(&self, relay: String) -> Result<Vec<String>, Error> {
        let core = self.core().clone();
        run_async(async move { core.nwc()?.add_relay(&relay).await }).await
    }

    /// Stop serving NWC connections on `relay`. Returns the relays after the
    /// change.
    ///
    /// An app whose URI names none of the remaining relays can no longer
    /// reach the wallet, and needs a new connection.
    pub async fn nwc_remove_relay(&self, relay: String) -> Result<Vec<String>, Error> {
        let core = self.core().clone();
        run_async(async move { core.nwc()?.remove_relay(&relay).await }).await
    }
}
