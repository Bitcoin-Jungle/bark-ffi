//! Nostr Wallet Connect (NIP-47) over wasm-bindgen. See [`crate::core::nwc`].
//!
//! Connections and relays are kept in an IndexedDB database of their own,
//! `<wallet database>.nwc`, rather than in a partition of the wallet's, whose
//! partition numbers are bark's to hand out.

#![allow(non_snake_case)]

use anyhow::Context;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, OnceCell};
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::*;

use bark::nwc::ConnectionStore;
use bark::persist::adaptor::indexed_db::IndexedDbClient;
use bark::persist::adaptor::{Record, StorageAdaptor};

use crate::core::nwc::NwcStore;
use crate::types::{NwcConnection, NwcConnectionUri, NwcMethod};
use crate::wasm::wallet::Wallet;

/// The one record the store is kept in.
const PARTITION: u8 = 0;
const KEY: &[u8] = b"connections";

/// The IndexedDB database NWC keeps its store in, next to the wallet's
/// `wallet_db`. Delete it along with the wallet's.
pub(crate) fn nwc_db_name(wallet_db: &str) -> String {
    format!("{}.nwc", wallet_db)
}

/// The NWC connection store in IndexedDB.
///
/// The database is opened on first use, so a failure to open it leaves NWC
/// unavailable rather than the wallet unopenable.
pub(crate) struct IdbNwcStore {
    name: String,
    db: OnceCell<Mutex<IndexedDbClient>>,
}

impl IdbNwcStore {
    pub(crate) fn new(name: String) -> Self {
        Self { name, db: OnceCell::new() }
    }

    async fn db(&self) -> anyhow::Result<&Mutex<IndexedDbClient>> {
        self.db.get_or_try_init(|| async {
            let db = IndexedDbClient::open(&self.name).await
                .with_context(|| format!("failed to open IndexedDB {}", self.name))?;
            Ok(Mutex::new(db))
        }).await
    }
}

#[async_trait::async_trait(?Send)]
impl NwcStore for IdbNwcStore {
    async fn load(&self) -> anyhow::Result<ConnectionStore> {
        let db = self.db().await?.lock().await;
        match db.get(PARTITION, KEY).await? {
            Some(record) => serde_json::from_slice(&record.data)
                .with_context(|| format!("failed to parse the NWC connections in {}", self.name)),
            None => Ok(ConnectionStore::new()),
        }
    }

    async fn save(&self, store: &ConnectionStore) -> anyhow::Result<()> {
        let data = serde_json::to_vec(store).context("failed to serialize NWC connections")?;
        let mut db = self.db().await?.lock().await;
        db.put(Record { partition: PARTITION, pk: KEY.to_vec(), sort_key: None, data }).await
    }
}

/// Arguments for [`Wallet::nwc_create_connection`].
#[derive(Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
pub struct NwcCreateConnectionArgs {
    /// Listed with the connection, to tell connections apart.
    pub name: String,
    /// The methods the app may call. `pay_invoice` lets it spend the whole
    /// balance, there is no spending limit.
    pub methods: Vec<NwcMethod>,
}

#[wasm_bindgen]
impl Wallet {
    // -- Nostr Wallet Connect -------------------------------------------------

    /// Create a Nostr Wallet Connect (NIP-47) connection that lets an app call
    /// `methods` on this wallet.
    ///
    /// Needs a relay, see `nwcAddRelay`. Returns the URI to hand to the app.
    /// It carries the app's secret key, which the wallet does not keep: show
    /// it once and never log or store it.
    ///
    /// Requests are answered while the daemon runs. A connection created while
    /// it is stopped is served once it starts.
    #[wasm_bindgen(js_name = nwcCreateConnection)]
    pub async fn nwc_create_connection(
        &self,
        args: Ts<NwcCreateConnectionArgs>,
    ) -> Result<Ts<NwcConnectionUri>, JsError> {
        let args = args.to_rust()?;
        let nwc = self.core().nwc()?;
        Ok(nwc.create_connection(args.name, args.methods).await?.into_ts()?)
    }

    /// Every NWC connection, oldest first.
    #[wasm_bindgen(js_name = nwcConnections)]
    pub fn nwc_connections(&self) -> Result<Vec<Ts<NwcConnection>>, JsError> {
        Ok(self.core().nwc()?.connections()
            .into_iter()
            .map(|c| c.into_ts())
            .collect::<Result<Vec<_>, _>>()?)
    }

    /// Revoke the NWC connection of the app with `clientPubkey` and stop
    /// serving it. Returns whether there was such a connection.
    #[wasm_bindgen(js_name = nwcRevokeConnection)]
    pub async fn nwc_revoke_connection(&self, clientPubkey: String) -> Result<bool, JsError> {
        Ok(self.core().nwc()?.revoke_connection(&clientPubkey).await?)
    }

    /// The relays NWC connections are served on.
    #[wasm_bindgen(js_name = nwcRelays)]
    pub fn nwc_relays(&self) -> Result<Vec<String>, JsError> {
        Ok(self.core().nwc()?.relays())
    }

    /// Serve NWC connections on `relay` too, a `ws://` or `wss://` URL.
    /// Returns the relays after the change.
    ///
    /// A connection's URI names the relays it was created with, so its app
    /// only finds the wallet on relays from that list.
    #[wasm_bindgen(js_name = nwcAddRelay)]
    pub async fn nwc_add_relay(&self, relay: String) -> Result<Vec<String>, JsError> {
        Ok(self.core().nwc()?.add_relay(&relay).await?)
    }

    /// Stop serving NWC connections on `relay`. Returns the relays after the
    /// change.
    ///
    /// An app whose URI names none of the remaining relays can no longer
    /// reach the wallet, and needs a new connection.
    #[wasm_bindgen(js_name = nwcRemoveRelay)]
    pub async fn nwc_remove_relay(&self, relay: String) -> Result<Vec<String>, JsError> {
        Ok(self.core().nwc()?.remove_relay(&relay).await?)
    }
}
