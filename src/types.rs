use bark::persist::models::PendingBoard as BarkPendingBoard;
use bark::WalletVtxo as BarkWalletVtxo;
use bark::{Balance as BarkBalance, WalletProperties as BarkWalletProperties};
use bark_bitcoin_ext::AmountExt;
use bitcoin::Network as BtcNetwork;
use serde::{Deserialize, Serialize};

// ============================================================================
// Network
// ============================================================================

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum Network {
    Bitcoin,
    Testnet,
    Signet,
    Regtest,
}

impl From<Network> for BtcNetwork {
    fn from(n: Network) -> Self {
        match n {
            Network::Bitcoin => BtcNetwork::Bitcoin,
            Network::Testnet => BtcNetwork::Testnet,
            Network::Signet => BtcNetwork::Signet,
            Network::Regtest => BtcNetwork::Regtest,
        }
    }
}

impl From<BtcNetwork> for Network {
    fn from(n: BtcNetwork) -> Self {
        match n {
            BtcNetwork::Bitcoin => Network::Bitcoin,
            BtcNetwork::Testnet => Network::Testnet,
            BtcNetwork::Testnet4 => Network::Testnet, // Map Testnet4 to Testnet
            BtcNetwork::Signet => Network::Signet,
            BtcNetwork::Regtest => Network::Regtest,
        }
    }
}

// ============================================================================
// Logging
// ============================================================================

// Serde/TS names are lowercase (`"error"` | `"warn"` | `"info"` | `"debug"` |
// `"trace"`), matching `as_str`. Kept out of the doc comment, which uniffi
// copies into the native bindings, where the level is a native enum.
/// Severity of a log record.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    /// The level's serde/TS name, e.g. `"warn"`.
    pub fn as_str(self) -> &'static str {
        match self {
            LogLevel::Error => "error",
            LogLevel::Warn => "warn",
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
            LogLevel::Trace => "trace",
        }
    }
}

impl From<log::Level> for LogLevel {
    fn from(l: log::Level) -> Self {
        match l {
            log::Level::Error => Self::Error,
            log::Level::Warn => Self::Warn,
            log::Level::Info => Self::Info,
            log::Level::Debug => Self::Debug,
            log::Level::Trace => Self::Trace,
        }
    }
}

impl From<LogLevel> for log::LevelFilter {
    fn from(l: LogLevel) -> Self {
        match l {
            LogLevel::Error => Self::Error,
            LogLevel::Warn => Self::Warn,
            LogLevel::Info => Self::Info,
            LogLevel::Debug => Self::Debug,
            LogLevel::Trace => Self::Trace,
        }
    }
}

// ============================================================================
// DebugInfo
// ============================================================================

/// Wallet information that helps to debug issues, mirroring
/// `bark::DebugInfo` (what `bark debug-info` prints). Users can share it in
/// support requests.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct DebugInfo {
    pub network: Network,
    /// The wallet's server mailbox id, hex-encoded like
    /// `Wallet::mailbox_identifier`.
    pub mailbox_id: String,
    /// The xpub every VTXO keypair is derived from.
    pub vtxo_xpub: String,
}

impl From<bark::DebugInfo> for DebugInfo {
    fn from(d: bark::DebugInfo) -> Self {
        Self {
            network: d.network.into(),
            // Displays as the hex public key, as `mailbox_identifier` encodes it.
            mailbox_id: d.mailbox_id.to_string(),
            vtxo_xpub: d.vtxo_xpub.to_string(),
        }
    }
}

// ============================================================================
// WalletProperties
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct WalletProperties {
    pub network: Network,
    pub fingerprint: String,
}

impl From<BarkWalletProperties> for WalletProperties {
    fn from(p: BarkWalletProperties) -> Self {
        Self {
            network: p.network.into(),
            fingerprint: p.fingerprint.to_string(),
        }
    }
}

// ============================================================================
// Balance
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct Balance {
    /// Sats that can be spent right now. Excludes `needs_refresh_sats`.
    pub spendable_sats: u64,
    /// Sats in VTXOs that have expired or reached the server's maximum exit
    /// depth. They can still be offboarded, exited or refreshed, but not sent
    /// out-of-round until maintenance has refreshed them.
    pub needs_refresh_sats: u64,
    pub pending_in_round_sats: u64,
    pub pending_exit_sats: u64,
    pub pending_lightning_send_sats: u64,
    pub claimable_lightning_receive_sats: u64,
    pub pending_board_sats: u64,
    /// Sats locked in an outgoing arkoor payment that has not completed yet.
    pub pending_arkoor_send_sats: u64,
    /// Sats locked in an offboard whose transaction has not been broadcast
    /// yet, including any change that comes back.
    pub pending_offboard_sats: u64,
}

/// The numbers to show a user without the breakdown of [`Balance`],
/// mirroring `bark::BalanceSummary`.
///
/// Show `spendable_sats` as what a user can pay with and `total_sats` as what
/// a user owns.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct BalanceSummary {
    /// Sats that can be spent right now.
    pub spendable_sats: u64,
    /// Sats that need a refresh before they can be sent again.
    pub needs_refresh_sats: u64,
    /// Sats held by operations in progress. They either come back as
    /// spendable or leave the wallet, for example to the onchain wallet.
    pub pending_sats: u64,
    /// All sats that belong to the wallet:
    /// `spendable_sats + needs_refresh_sats + pending_sats`.
    pub total_sats: u64,
}

impl From<bark::BalanceSummary> for BalanceSummary {
    fn from(s: bark::BalanceSummary) -> Self {
        Self {
            spendable_sats: s.spendable.to_sat(),
            needs_refresh_sats: s.needs_refresh.to_sat(),
            pending_sats: s.pending.to_sat(),
            total_sats: s.total.to_sat(),
        }
    }
}

impl From<BarkBalance> for Balance {
    fn from(b: BarkBalance) -> Self {
        Self {
            spendable_sats: b.spendable.to_sat(),
            needs_refresh_sats: b.needs_refresh.to_sat(),
            pending_in_round_sats: b.pending_in_round.to_sat(),
            pending_exit_sats: b.pending_exit.to_sat(),
            pending_lightning_send_sats: b.pending_lightning_send.to_sat(),
            // Note: Bark's Balance only has claimable_lightning_receive field.
            // There is no separate "total pending receive" field in upstream.
            claimable_lightning_receive_sats: b.claimable_lightning_receive.to_sat(),
            pending_board_sats: b.pending_board.to_sat(),
            pending_arkoor_send_sats: b.pending_arkoor_send.to_sat(),
            pending_offboard_sats: b.pending_offboard.to_sat(),
        }
    }
}

// ============================================================================
// Vtxo
// ============================================================================

/// Who holds the lock on a [`VtxoState::Locked`] VTXO.
///
/// Mirrors `bark::vtxo::VtxoLockHolder`. Action-based subsystems lock with
/// `Action`; pre-action subsystems (round, offboard, board, lightning
/// receive) lock with `Movement`. As upstream converts subsystems to
/// actions, new locks migrate from `Movement` to `Action` per-subsystem.
///
/// Serde/TS tags match upstream's serialization: `"action"` / `"movement"`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "kebab-case")]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    // Both directions: read back off `Vtxo.state`, and passed in when
    // locking or unlocking.
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum VtxoLockHolder {
    /// A wallet action checkpoint. `id` is upstream's `WalletActionId`
    /// (an opaque string).
    Action { id: String },
    /// A pre-action subsystem, keyed by its movement.
    Movement { id: u32 },
    /// Scuttle: a VTXO received by arkoor that a server in expiry payout
    /// mode does not report spendable yet. Credited once the server does.
    MailboxReceive,
    /// Scuttle: a round output that a server in expiry payout mode does not
    /// report bound to this wallet's mailbox yet. Released once it does.
    RoundPayoutCheck,
}

impl From<&bark::vtxo::VtxoLockHolder> for VtxoLockHolder {
    fn from(h: &bark::vtxo::VtxoLockHolder) -> Self {
        match h {
            bark::vtxo::VtxoLockHolder::Action { id } => Self::Action { id: id.clone() },
            bark::vtxo::VtxoLockHolder::Movement { id } => Self::Movement { id: id.0 },
            bark::vtxo::VtxoLockHolder::MailboxReceive => Self::MailboxReceive,
            bark::vtxo::VtxoLockHolder::RoundPayoutCheck => Self::RoundPayoutCheck,
        }
    }
}

impl From<VtxoLockHolder> for bark::vtxo::VtxoLockHolder {
    fn from(h: VtxoLockHolder) -> Self {
        match h {
            VtxoLockHolder::Action { id } => Self::Action { id },
            VtxoLockHolder::Movement { id } => Self::Movement {
                id: bark::movement::MovementId::new(id),
            },
            VtxoLockHolder::MailboxReceive => Self::MailboxReceive,
            VtxoLockHolder::RoundPayoutCheck => Self::RoundPayoutCheck,
        }
    }
}

/// Rich VTXO state, mirroring `bark::vtxo::VtxoState`.
///
/// Serde/TS tags match upstream's kebab-case serialization:
/// `"spendable"` | `"locked"` | `"spent"` | `"exited"`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "kebab-case")]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum VtxoState {
    /// Available; can be spent in a future round.
    Spendable,
    /// Locked by an operation. `holder` is `None` only for the narrow
    /// window between creating a fresh locked VTXO and pinning it to a
    /// specific operation, so a locked VTXO can legitimately carry no
    /// lock reason yet.
    Locked { holder: Option<VtxoLockHolder> },
    /// Consumed.
    Spent,
    /// In (or completed) a unilateral exit.
    Exited,
}

impl From<&bark::vtxo::VtxoState> for VtxoState {
    fn from(s: &bark::vtxo::VtxoState) -> Self {
        match s {
            bark::vtxo::VtxoState::Spendable => Self::Spendable,
            bark::vtxo::VtxoState::Locked { holder } => Self::Locked {
                holder: holder.as_ref().map(Into::into),
            },
            bark::vtxo::VtxoState::Spent => Self::Spent,
            bark::vtxo::VtxoState::Exited => Self::Exited,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct Vtxo {
    pub id: String,
    pub amount_sats: u64,
    pub expiry_height: u32,
    pub kind: String,
    pub state: VtxoState,
    /// Genesis chain length. Compare against `ArkInfo.max_vtxo_exit_depth` to
    /// detect VTXOs nearing the server's OOR-cosign refusal threshold.
    pub exit_depth: u32,
    /// Weight units of the unilateral exit transaction chain. Lets clients
    /// estimate exit cost without loading the full genesis.
    pub exit_tx_weight_wu: u64,
    /// Whether this VTXO's recovery state has been asserted with the server:
    /// its id posted to the recovery mailbox and its signed transaction chain
    /// registered. Only ever moves from `false` to `true`; the sync-time
    /// catch-up re-uploads the ones still `false`.
    pub registered: bool,
}

impl From<BarkWalletVtxo> for Vtxo {
    fn from(v: BarkWalletVtxo) -> Self {
        Self {
            id: v.vtxo.id().to_string(),
            amount_sats: v.vtxo.amount().to_sat(),
            expiry_height: v.vtxo.expiry_height().to_u32(),
            kind: format!("{:?}", v.vtxo.policy_type()),
            state: (&v.state).into(),
            exit_depth: v.exit_depth as u32,
            exit_tx_weight_wu: v.exit_tx_weight.to_wu(),
            registered: v.registered,
        }
    }
}

// ============================================================================
// LightningInvoice
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct LightningInvoice {
    pub invoice: String,
    pub payment_hash: String,
    pub amount_sats: u64,
}

// ============================================================================
// OffboardResult
// ============================================================================

/// What `offboard_all`, `offboard_vtxos` and `send_onchain` broadcast,
/// mirroring `bark::OffboardResult`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct OffboardResult {
    /// The offboard transaction the server built and bark broadcast.
    pub txid: String,
    /// The movement that records this offboard.
    pub movement_id: u32,
}

impl From<bark::OffboardResult> for OffboardResult {
    fn from(r: bark::OffboardResult) -> Self {
        Self {
            txid: r.txid.to_string(),
            movement_id: r.movement_id.0,
        }
    }
}

// ============================================================================
// AddressWithIndex
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct AddressWithIndex {
    pub address: String,
    pub index: u32,
}

// ============================================================================
// LightningReceive
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct LightningReceive {
    pub payment_hash: String,
    pub invoice: String,
    /// `None` for an amountless invoice that has not settled yet. It used to
    /// report 0, which a UI cannot tell from a genuine zero.
    pub amount_sats: Option<u64>,
    /// Receive progress: "awaiting-payment" | "htlcs-ready" |
    /// "preimage-revealed" | "delivering" | "settled"
    pub state: String,
    /// Known while in-progress; present when settled.
    pub payment_preimage: Option<String>,
    /// Unix timestamp (seconds), set only when the receive is settled.
    pub settled_at: Option<i64>,
    /// Ark address the claimed VTXO is delivered to, for receives created with
    /// `bolt11_invoice_for_address`. `None` for ordinary receives claimed by
    /// this wallet, and always `None` once settled — the settled record does
    /// not carry the destination.
    pub claim_destination: Option<String>,
}

impl From<bark::actions::lightning::receive::LightningReceive> for LightningReceive {
    fn from(r: bark::actions::lightning::receive::LightningReceive) -> Self {
        use bark::actions::lightning::receive::Progress;
        let state = match r.progress {
            Progress::AwaitingPayment => "awaiting-payment",
            Progress::HtlcsReady(_) => "htlcs-ready",
            Progress::PreimageRevealed(_) => "preimage-revealed",
            Progress::Delivering(_) => "delivering",
        };
        Self {
            payment_hash: r.payment_hash.to_string(),
            invoice: r.invoice.to_string(),
            amount_sats: r
                .invoice
                .amount_milli_satoshis()
                .map(|a| bitcoin::Amount::from_msat_floor(a).to_sat()),
            state: state.to_string(),
            payment_preimage: Some(r.payment_preimage.to_string()),
            settled_at: None,
            claim_destination: r.claim_destination.map(|a| a.to_string()),
        }
    }
}

impl From<bark::persist::models::SettledLightningReceive> for LightningReceive {
    fn from(r: bark::persist::models::SettledLightningReceive) -> Self {
        Self {
            payment_hash: r.payment_hash.to_string(),
            invoice: r.invoice.to_string(),
            amount_sats: Some(r.amount.to_sat()),
            state: "settled".to_string(),
            payment_preimage: Some(r.preimage.to_string()),
            settled_at: Some(r.settled_at.timestamp()),
            // The settled persist model doesn't carry the claim destination.
            claim_destination: None,
        }
    }
}

impl From<bark::actions::lightning::receive::LightningReceiveState> for LightningReceive {
    fn from(s: bark::actions::lightning::receive::LightningReceiveState) -> Self {
        use bark::actions::lightning::receive::LightningReceiveState;
        match s {
            LightningReceiveState::InProgress(r) => r.into(),
            LightningReceiveState::Settled(r) => r.into(),
        }
    }
}

// ============================================================================
// LightningSend
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct LightningSend {
    pub invoice: String,
    /// Payment hash of the invoice being paid. Poll `lightning_send_state`
    /// with it; for BOLT12 there is no invoice string to parse it from.
    pub payment_hash: String,
    /// Id of the `Movement` that records this send. `None` only for sends
    /// checkpointed by a bark version that predates the field.
    pub movement_id: Option<u32>,
    /// Amount being paid, in sats (`payment_amount`).
    pub amount_sats: u64,
    /// Routing/ark fee for the send, in sats.
    pub fee_sats: u64,
    /// Number of input VTXOs locked into the in-flight HTLC.
    pub htlc_vtxo_count: u32,
    /// Whether the send is stuck in the revocation-failed state: the payment
    /// failed but revoking the HTLC also failed
    pub has_failed_revocation: bool,
}

impl From<bark::actions::lightning::pay::LightningSend> for LightningSend {
    fn from(s: bark::actions::lightning::pay::LightningSend) -> Self {
        Self {
            payment_hash: s.invoice.payment_hash().to_string(),
            movement_id: s.movement_id.map(|m| m.0),
            invoice: s.invoice.to_string(),
            amount_sats: s.payment_amount.to_sat(),
            fee_sats: s.fee.to_sat(),
            htlc_vtxo_count: s.input_vtxo_ids.len() as u32,
            has_failed_revocation: s.has_failed_revocation(),
        }
    }
}

// ============================================================================
// LightningSendStatus
// ============================================================================

/// Terminal/in-flight state of an outgoing lightning send.
///
/// Mirrors `bark`'s `LightningSendState`. `pay_lightning_*` and
/// `check_lightning_payment` now return this so callers can drive the
/// crash-safe send flow themselves (initiate with `wait = false`, then poll).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(tag = "type", rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum LightningSendStatus {
    /// No record of this payment (never started, or already pruned).
    Unknown,
    /// Send is in flight; HTLC VTXOs are locked.
    InProgress { send: LightningSend },
    /// Send has settled; the preimage proves payment.
    Paid {
        payment_hash: String,
        preimage: String,
    },
}

impl From<bark::actions::lightning::pay::LightningSendState> for LightningSendStatus {
    fn from(state: bark::actions::lightning::pay::LightningSendState) -> Self {
        use bark::actions::lightning::pay::LightningSendState as State;
        match state {
            State::Unknown => Self::Unknown,
            State::InProgress(send) => Self::InProgress { send: send.into() },
            State::Paid(paid) => Self::Paid {
                payment_hash: paid.payment_hash.to_string(),
                preimage: paid.preimage.to_string(),
            },
        }
    }
}

// ============================================================================
// Movement
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct Movement {
    pub id: u32,
    pub status: String,
    pub subsystem_name: String,
    pub subsystem_kind: String,
    pub metadata_json: String,
    pub intended_balance_sats: i64,
    pub effective_balance_sats: i64,
    pub offchain_fee_sats: u64,
    pub sent_to_addresses: Vec<String>,
    pub received_on_addresses: Vec<String>,
    pub input_vtxo_ids: Vec<String>,
    pub output_vtxo_ids: Vec<String>,
    pub exited_vtxo_ids: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,

    // calculated fields
    pub payment_hash: Option<String>,
    pub lightning_invoice: Option<String>,
    pub lightning_offer: Option<String>,
}

impl From<bark::movement::Movement> for Movement {
    fn from(m: bark::movement::Movement) -> Self {
        Self {
            id: m.id.0,
            status: m.status.to_string(),
            subsystem_name: m.subsystem.name.clone(),
            subsystem_kind: m.subsystem.kind.clone(),
            metadata_json: serde_json::to_string(&m.metadata).unwrap_or_else(|_| "{}".to_string()),
            intended_balance_sats: m.intended_balance.to_sat(),
            effective_balance_sats: m.effective_balance.to_sat(),
            offchain_fee_sats: m.offchain_fee.to_sat(),
            sent_to_addresses: m
                .sent_to
                .iter()
                .map(|d| {
                    // Serialize PaymentMethod to JSON string for FFI
                    serde_json::to_string(&d.destination)
                        .unwrap_or_else(|_| format!("{:?}", d.destination))
                })
                .collect(),
            received_on_addresses: m
                .received_on
                .iter()
                .map(|d| {
                    // Serialize PaymentMethod to JSON string for FFI
                    serde_json::to_string(&d.destination)
                        .unwrap_or_else(|_| format!("{:?}", d.destination))
                })
                .collect(),
            input_vtxo_ids: m.input_vtxos.iter().map(|v| v.to_string()).collect(),
            output_vtxo_ids: m.output_vtxos.iter().map(|v| v.to_string()).collect(),
            exited_vtxo_ids: m.exited_vtxos.iter().map(|v| v.to_string()).collect(),
            created_at: m.time.created_at.to_rfc3339(),
            updated_at: m.time.updated_at.to_rfc3339(),
            completed_at: m.time.completed_at.map(|t| t.to_rfc3339()),

            payment_hash: m.lightning_payment_hash().map(|h| h.to_string()),
            lightning_invoice: m.lightning_invoice().map(|i| i.to_string()),
            lightning_offer: m.lightning_offer().map(|o| o.to_string()),
        }
    }
}

// ============================================================================
// FeeEstimate
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct FeeEstimate {
    pub gross_amount_sats: u64,
    pub fee_sats: u64,
    pub net_amount_sats: u64,
    pub vtxos_spent: Vec<String>,
}

impl From<bark::FeeEstimate> for FeeEstimate {
    fn from(f: bark::FeeEstimate) -> Self {
        Self {
            gross_amount_sats: f.gross_amount.to_sat(),
            fee_sats: f.fee.to_sat(),
            net_amount_sats: f.net_amount.to_sat(),
            vtxos_spent: f.vtxos_spent.iter().map(|v| v.to_string()).collect(),
        }
    }
}

// ============================================================================
// EmergencyExitFeeEstimate
// ============================================================================

/// Estimated onchain cost of unilaterally (emergency) exiting a set of VTXOs.
///
/// Mirrors bark-rest's `GET /exits/fee` response. The two legs are paid from
/// different pockets: `exit_broadcast_fee_sats` is spent now out of confirmed
/// onchain funds to CPFP-bump every not-yet-confirmed exit transaction, while
/// `claim_fee_sats` is deducted later from the recovered value when the
/// matured outputs are drained.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct EmergencyExitFeeEstimate {
    /// CPFP fees to broadcast every not-yet-confirmed exit transaction,
    /// scaled by the fee margin, so deliberately above `fee_rate × weight`.
    /// Paid now from confirmed onchain funds; this is the onchain balance to
    /// fund the exit with at that margin.
    pub exit_broadcast_fee_sats: u64,
    /// Fee of the single batched transaction that later drains the matured
    /// exit outputs. Subtracted from the exited amount.
    pub claim_fee_sats: u64,
    /// `exit_broadcast_fee_sats + claim_fee_sats`.
    pub total_fee_sats: u64,
    /// Fee rate the exit-broadcast leg was priced at, before the fee margin
    /// (sat/vB, rounded up). Unless an explicit rate was supplied, the claim
    /// leg is priced separately at the chain's `regular` rate.
    pub fee_rate_sat_per_vb: u64,
    /// Number of exit transactions that still need to be broadcast and
    /// CPFP-bumped. Already-confirmed tree transactions are not counted.
    pub txs_to_broadcast: u64,
    /// Whether the onchain wallet's last synced balance covers
    /// `exit_broadcast_fee_sats`. `false` without an onchain wallet.
    ///
    /// Deprecated: bark dropped this field. Compare `exit_broadcast_fee_sats`
    /// against the onchain balance instead; this one goes away in a later
    /// release, as it does in bark-rest.
    pub fundable: bool,
}

impl EmergencyExitFeeEstimate {
    /// Builds the estimate from bark's and the onchain balance, which only
    /// feeds the deprecated `fundable`.
    pub(crate) fn new(e: bark::exit::ExitFeeEstimate, onchain_balance: Option<bitcoin::Amount>) -> Self {
        Self {
            total_fee_sats: e.total().to_sat(),
            exit_broadcast_fee_sats: e.exit_broadcast_fee.to_sat(),
            claim_fee_sats: e.claim_fee.to_sat(),
            fee_rate_sat_per_vb: e.fee_rate.to_sat_per_vb_ceil(),
            txs_to_broadcast: e.txs_to_broadcast as u64,
            fundable: onchain_balance.is_some_and(|b| b >= e.exit_broadcast_fee),
        }
    }
}

// ============================================================================
// OnchainBalance
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct OnchainBalance {
    pub confirmed_sats: u64,
    pub pending_sats: u64,
    pub total_sats: u64,
}

impl From<bark::onchain::bdk_wallet::Balance> for OnchainBalance {
    fn from(b: bark::onchain::bdk_wallet::Balance) -> Self {
        Self {
            confirmed_sats: b.confirmed.to_sat(),
            pending_sats: b.untrusted_pending.to_sat() + b.trusted_pending.to_sat(),
            total_sats: b.total().to_sat(),
        }
    }
}

// ============================================================================
// Onchain wallet data (transactions, UTXOs, fee rates)
// ============================================================================

/// Network fee rates by urgency, mirroring `bark::chain::FeeRates`.
///
/// Rates are in sat/kwu (satoshis per 1000 weight units) — divide by 250 for
/// sat/vB.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct FeeRates {
    pub fast_sat_per_kwu: u64,
    pub regular_sat_per_kwu: u64,
    pub slow_sat_per_kwu: u64,
}

impl From<bark::chain::FeeRates> for FeeRates {
    fn from(f: bark::chain::FeeRates) -> Self {
        Self {
            fast_sat_per_kwu: f.fast.to_sat_per_kwu(),
            regular_sat_per_kwu: f.regular.to_sat_per_kwu(),
            slow_sat_per_kwu: f.slow.to_sat_per_kwu(),
        }
    }
}

/// Summary of one onchain wallet transaction, mirroring
/// `bark::onchain::WalletTxInfo`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct WalletTransaction {
    pub txid: String,
    /// The raw transaction, consensus-serialized as hex.
    pub tx_hex: String,
    /// Total fee paid by the transaction, when computable. `None` for
    /// inbound or collaboratively-funded txs whose foreign prevouts the
    /// wallet has not indexed.
    pub onchain_fee_sats: Option<u64>,
    /// Net change to the wallet's balance: received minus sent over
    /// wallet-owned outputs.
    pub balance_change_sats: i64,
    /// `Some` if confirmed in a block, `None` if still in the mempool.
    pub confirmation: Option<BlockRef>,
    /// `true` when this tx spends a P2A fee anchor — i.e. it is a CPFP
    /// child bumping the parent that created the anchor (exit fee txs).
    pub is_cpfp: bool,
}

impl From<&bark::onchain::WalletTxInfo> for WalletTransaction {
    fn from(info: &bark::onchain::WalletTxInfo) -> Self {
        Self {
            txid: info.txid.to_string(),
            tx_hex: bitcoin::consensus::encode::serialize_hex(info.tx.as_ref()),
            onchain_fee_sats: info.onchain_fees.map(|a| a.to_sat()),
            balance_change_sats: info.balance_change.to_sat(),
            confirmation: info.confirmation.as_ref().map(Into::into),
            is_cpfp: info.is_cpfp,
        }
    }
}

/// An onchain UTXO known to the wallet, mirroring `bark::onchain::Utxo`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case", rename_all_fields = "camelCase")]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum OnchainUtxo {
    /// A standard wallet UTXO.
    Local {
        outpoint: OutPoint,
        amount_sats: u64,
        /// `None` if unconfirmed.
        confirmation_height: Option<u32>,
    },
    /// A spendable unilateral-exit output claimed from a VTXO.
    Exit {
        vtxo_id: String,
        amount_sats: u64,
        /// Block height associated with the exit's validity window.
        height: u32,
    },
}

impl From<&bark::onchain::Utxo> for OnchainUtxo {
    fn from(u: &bark::onchain::Utxo) -> Self {
        match u {
            bark::onchain::Utxo::Local(l) => Self::Local {
                outpoint: (&l.outpoint).into(),
                amount_sats: l.amount.to_sat(),
                confirmation_height: l.confirmation_height.map(|h| h.to_u32()),
            },
            bark::onchain::Utxo::Exit(e) => Self::Exit {
                vtxo_id: e.vtxo.id().to_string(),
                amount_sats: e.vtxo.amount().to_sat(),
                height: e.height.to_u32(),
            },
        }
    }
}

// ============================================================================
// PendingBoard
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PendingBoard {
    pub vtxo_id: String,
    pub amount_sats: u64,
    pub txid: String,
    /// Id of the `Movement` that records this board.
    pub movement_id: u32,
}

impl From<BarkPendingBoard> for PendingBoard {
    fn from(pb: BarkPendingBoard) -> Self {
        Self {
            vtxo_id: pb.vtxos.first().map(|v| v.to_string()).unwrap_or_default(),
            amount_sats: pb.amount.to_sat(),
            txid: pb.funding_tx.compute_txid().to_string(),
            movement_id: pb.movement_id.0,
        }
    }
}

// ============================================================================
// BoardFundingInfo
// ============================================================================

/// Destination for an externally funded board (e.g. a payjoin receive),
/// mirroring `bark::BoardFundingAddress`.
///
/// The funding script commits to `user_pubkey` and `expiry_height`, so persist
/// this record and pass it back unchanged to `board_psbt` once the funding
/// PSBT is available — including on retries after the sender changes the
/// transaction. Fund it right away: the board VTXO's lifetime started when
/// the address was made, and once it expires the server can sweep the funding
/// output alone.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct BoardFundingInfo {
    /// Address the funding transaction must pay. Informational only:
    /// `board_psbt` rebuilds the funding script from `user_pubkey` and
    /// `expiry_height` and does not read it.
    pub address: String,
    /// Block height at which the board VTXO expires.
    pub expiry_height: u32,
    /// Hex-encoded wallet public key the board VTXO is locked to.
    pub user_pubkey: String,
}

/// `bark::BoardFundingAddress` keeps its fields private and only exposes them
/// through serde, so convert through its serialized form.
#[derive(Serialize, Deserialize)]
struct BarkBoardFundingAddress {
    address: String,
    user_pubkey: String,
    expiry_height: u32,
}

impl TryFrom<&bark::BoardFundingAddress> for BoardFundingInfo {
    type Error = crate::error::Error;

    fn try_from(f: &bark::BoardFundingAddress) -> Result<Self, Self::Error> {
        let value = serde_json::to_value(f)
            .map_err(|e| format!("failed to encode board funding address: {}", e))?;
        let raw: BarkBoardFundingAddress = serde_json::from_value(value)
            .map_err(|e| format!("failed to decode board funding address: {}", e))?;
        Ok(Self {
            address: raw.address,
            expiry_height: raw.expiry_height,
            user_pubkey: raw.user_pubkey,
        })
    }
}

impl TryFrom<BoardFundingInfo> for bark::BoardFundingAddress {
    type Error = crate::error::Error;

    fn try_from(f: BoardFundingInfo) -> Result<Self, Self::Error> {
        let raw = BarkBoardFundingAddress {
            address: f.address,
            user_pubkey: f.user_pubkey,
            expiry_height: f.expiry_height,
        };
        let value = serde_json::to_value(raw)
            .map_err(|e| format!("failed to encode board funding info: {}", e))?;
        Ok(serde_json::from_value(value)
            .map_err(|e| format!("invalid board funding info: {}", e))?)
    }
}

// ============================================================================
// FeeSchedule
// ============================================================================

/// One tier of a PPM-by-expiry fee table.
///
/// The entry applies when a VTXO expires in at most `expiry_blocks_threshold`
/// blocks and no other entry has a threshold between this one and the VTXO's
/// actual expiry distance. Tables are sorted ascending by threshold.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PpmExpiryFeeEntry {
    pub expiry_blocks_threshold: u32,
    /// Parts-per-million fee rate applied for this expiry period.
    pub ppm: u64,
}

/// Fees for boarding onchain funds into the Ark.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct BoardFees {
    pub min_fee_sats: u64,
    pub base_fee_sats: u64,
    /// Parts-per-million fee rate on the boarded amount.
    pub ppm: u64,
}

/// Fees for offboarding VTXOs to an onchain address.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct OffboardFees {
    pub base_fee_sats: u64,
    /// Fixed number of virtual bytes charged on top of the output size.
    pub fixed_additional_vb: u64,
    pub ppm_expiry_table: Vec<PpmExpiryFeeEntry>,
}

/// Fees for refreshing VTXOs in a round.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct RefreshFees {
    pub base_fee_sats: u64,
    pub ppm_expiry_table: Vec<PpmExpiryFeeEntry>,
}

/// Fees for receiving over lightning.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct LightningReceiveFees {
    pub base_fee_sats: u64,
    /// Parts-per-million fee rate on the received amount.
    pub ppm: u64,
}

/// Fees for sending over lightning.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct LightningSendFees {
    pub min_fee_sats: u64,
    pub base_fee_sats: u64,
    pub ppm_expiry_table: Vec<PpmExpiryFeeEntry>,
}

/// The Ark server's complete fee schedule, mirroring `ark::fees::FeeSchedule`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct FeeSchedule {
    pub board: BoardFees,
    pub offboard: OffboardFees,
    pub refresh: RefreshFees,
    pub lightning_receive: LightningReceiveFees,
    pub lightning_send: LightningSendFees,
    /// Scuttle: the Lightning fees of a payment between two wallets of this
    /// server, which never leaves it, in place of `lightning_send` and
    /// `lightning_receive`. `None` when the server charges those as usual.
    pub intra_ark: Option<IntraArkFees>,
}

/// Scuttle: same-server Lightning fees, mirroring `ark::fees::IntraArkFees`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct IntraArkFees {
    /// Charged to the payer in place of `FeeSchedule::lightning_send`.
    pub lightning_send: LightningSendFees,
    /// Charged to the receiver in place of `FeeSchedule::lightning_receive`.
    pub lightning_receive: LightningReceiveFees,
}

impl From<&ark::fees::FeeSchedule> for FeeSchedule {
    fn from(f: &ark::fees::FeeSchedule) -> Self {
        let table = |t: &[ark::fees::PpmExpiryFeeEntry]| {
            t.iter()
                .map(|e| PpmExpiryFeeEntry {
                    expiry_blocks_threshold: e.expiry_blocks_threshold,
                    ppm: e.ppm.0,
                })
                .collect()
        };
        let receive = |r: &ark::fees::LightningReceiveFees| LightningReceiveFees {
            base_fee_sats: r.base_fee.to_sat(),
            ppm: r.ppm.0,
        };
        let send = |l: &ark::fees::LightningSendFees| LightningSendFees {
            min_fee_sats: l.min_fee.to_sat(),
            base_fee_sats: l.base_fee.to_sat(),
            ppm_expiry_table: table(&l.ppm_expiry_table),
        };
        Self {
            board: BoardFees {
                min_fee_sats: f.board.min_fee.to_sat(),
                base_fee_sats: f.board.base_fee.to_sat(),
                ppm: f.board.ppm.0,
            },
            offboard: OffboardFees {
                base_fee_sats: f.offboard.base_fee.to_sat(),
                fixed_additional_vb: f.offboard.fixed_additional_vb,
                ppm_expiry_table: table(&f.offboard.ppm_expiry_table),
            },
            refresh: RefreshFees {
                base_fee_sats: f.refresh.base_fee.to_sat(),
                ppm_expiry_table: table(&f.refresh.ppm_expiry_table),
            },
            lightning_receive: receive(&f.lightning_receive),
            lightning_send: send(&f.lightning_send),
            intra_ark: f.intra_ark.as_ref().map(|i| IntraArkFees {
                lightning_send: send(&i.lightning_send),
                lightning_receive: receive(&i.lightning_receive),
            }),
        }
    }
}

// ============================================================================
// ArkInfo
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct ArkInfo {
    pub network: Network,
    pub server_pubkey: String,
    pub round_interval_secs: u64,
    pub nb_round_nonces: u32,
    pub vtxo_exit_delta: u32,
    /// The number of blocks a VTXO lives before it expires.
    pub vtxo_lifetime: u32,
    /// The number of blocks a VTXO lives before it expires.
    ///
    /// Deprecated: upstream renamed this to `vtxo_lifetime`. Both names are
    /// exposed and carry the same value; migrate to `vtxo_lifetime`, this one
    /// goes away once upstream drops it.
    pub vtxo_expiry_delta: u32,
    pub htlc_send_expiry_delta: u32,
    pub htlc_expiry_delta: u32,
    pub max_vtxo_amount_sats: Option<u64>,
    pub required_board_confirmations: u32,
    pub max_user_invoice_cltv_delta: u16,
    pub min_board_amount_sats: u64,
    pub ln_receive_anti_dos_required: bool,
    /// The server's fee schedule (board, offboard, refresh, lightning fees).
    pub fee_schedule: FeeSchedule,
    /// Maximum exit depth (genesis chain length) allowed for a VTXO before the
    /// server refuses to cosign further OOR transactions spending it.
    pub max_vtxo_exit_depth: u16,
    /// Maximum number of outputs of a single round participation.
    pub max_round_participation_outputs: u32,
}

impl From<&bark::ark::ArkInfo> for ArkInfo {
    fn from(info: &bark::ark::ArkInfo) -> Self {
        use bitcoin::hex::DisplayHex;

        Self {
            network: match info.network {
                BtcNetwork::Bitcoin => Network::Bitcoin,
                BtcNetwork::Testnet => Network::Testnet,
                BtcNetwork::Signet => Network::Signet,
                BtcNetwork::Regtest => Network::Regtest,
                _ => Network::Testnet,
            },
            server_pubkey: info.server_pubkey.serialize().as_hex().to_string(),
            round_interval_secs: info.round_interval.as_secs(),
            nb_round_nonces: info.nb_round_nonces as u32,
            vtxo_exit_delta: info.vtxo_exit_delta.to_u32(),
            vtxo_lifetime: info.vtxo_lifetime.to_u32(),
            vtxo_expiry_delta: info.vtxo_lifetime.to_u32(),
            htlc_send_expiry_delta: info.htlc_send_expiry_delta.to_u32(),
            htlc_expiry_delta: info.htlc_expiry_delta.to_u32(),
            max_vtxo_amount_sats: info.max_vtxo_amount.map(|a| a.to_sat()),
            required_board_confirmations: info.required_board_confirmations as u32,
            max_user_invoice_cltv_delta: info.max_user_invoice_cltv_delta.to_u16(),
            min_board_amount_sats: info.min_board_amount.to_sat(),
            ln_receive_anti_dos_required: info.ln_receive_anti_dos_required,
            fee_schedule: (&info.fees).into(),
            max_vtxo_exit_depth: info.max_vtxo_exit_depth,
            max_round_participation_outputs: u32::try_from(info.max_round_participation_outputs)
                .unwrap_or(u32::MAX),
        }
    }
}

// ============================================================================
// Exit Types
// ============================================================================

use bark::exit::ExitProgressStatus as BarkExitProgressStatus;
use bark::exit::ExitVtxo as BarkExitVtxo;

/// Where an exit transaction was first seen, mirroring `bark::exit::ExitTxOrigin`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "kebab-case", rename_all_fields = "camelCase")]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum ExitTxOrigin {
    /// Broadcast by this wallet.
    Wallet { confirmed_in: Option<BlockRef> },
    /// Seen in the mempool.
    Mempool,
    /// Seen confirmed in a block.
    Block { confirmed_in: BlockRef },
}

impl From<&bark::exit::ExitTxOrigin> for ExitTxOrigin {
    fn from(o: &bark::exit::ExitTxOrigin) -> Self {
        use bark::exit::ExitTxOrigin as B;
        match o {
            B::Wallet { confirmed_in } => Self::Wallet {
                confirmed_in: confirmed_in.as_ref().map(Into::into),
            },
            B::Mempool => Self::Mempool,
            B::Block { confirmed_in } => Self::Block {
                confirmed_in: confirmed_in.into(),
            },
        }
    }
}

/// Broadcast/confirmation status of one transaction in an exit chain,
/// mirroring `bark::exit::ExitTxStatus`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "kebab-case", rename_all_fields = "camelCase")]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum ExitTxStatus {
    /// Inputs are still being verified.
    VerifyInputs,
    /// Waiting for the given input txids to confirm. Sorted for a stable
    /// order (upstream keeps them in a set).
    AwaitingInputConfirmation { txids: Vec<String> },
    /// Ready for its CPFP child to be broadcast.
    AwaitingCpfpBroadcast,
    /// CPFP child broadcast; waiting for confirmation.
    AwaitingConfirmation {
        child_txid: String,
        origin: ExitTxOrigin,
    },
    /// Confirmed in a block.
    Confirmed {
        /// Deprecated: a confirmed exit tx no longer tracks its CPFP child.
        /// Always empty, kept for compatibility.
        child_txid: String,
        /// The block that confirmed the exit tx.
        block: BlockRef,
        /// Deprecated: use `block`. Always a `Block` origin confirmed in
        /// `block`, kept for compatibility.
        origin: ExitTxOrigin,
    },
}

impl From<&bark::exit::ExitTxStatus> for ExitTxStatus {
    fn from(s: &bark::exit::ExitTxStatus) -> Self {
        use bark::exit::ExitTxStatus as B;
        match s {
            B::VerifyInputs => Self::VerifyInputs,
            B::AwaitingInputConfirmation { txids } => {
                let mut txids: Vec<String> = txids.iter().map(|t| t.to_string()).collect();
                txids.sort_unstable();
                Self::AwaitingInputConfirmation { txids }
            }
            B::AwaitingCpfpBroadcast => Self::AwaitingCpfpBroadcast,
            B::AwaitingConfirmation { child_txid, origin } => Self::AwaitingConfirmation {
                child_txid: child_txid.to_string(),
                origin: origin.into(),
            },
            B::Confirmed { block } => {
                let block = BlockRef::from(block);
                Self::Confirmed {
                    child_txid: String::new(),
                    block: block.clone(),
                    origin: ExitTxOrigin::Block { confirmed_in: block },
                }
            }
        }
    }
}

/// One transaction in an exit's unilateral broadcast chain.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct ExitTx {
    pub txid: String,
    pub status: ExitTxStatus,
}

impl From<&bark::exit::ExitTx> for ExitTx {
    fn from(t: &bark::exit::ExitTx) -> Self {
        Self {
            txid: t.txid.to_string(),
            status: (&t.status).into(),
        }
    }
}

/// State of a unilateral exit, mirroring `bark::exit::ExitState`.
///
/// Serde/TS tags match upstream's kebab-case serialization (`"start"`,
/// `"processing"`, `"awaiting-delta"`, `"claimable"`, `"claim-in-progress"`,
/// `"claimed"`, `"vtxo-already-spent"`, `"vtxo-swept"`, `"canceled"`), with
/// camelCase fields.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "kebab-case", rename_all_fields = "camelCase")]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum ExitState {
    /// The exit was requested at the given tip.
    Start { tip_height: u32 },
    /// The exit transaction chain is being broadcast and confirmed.
    Processing {
        tip_height: u32,
        transactions: Vec<ExitTx>,
    },
    /// Fully confirmed; waiting out the exit delta until claimable.
    AwaitingDelta {
        tip_height: u32,
        confirmed_block: BlockRef,
        claimable_height: u32,
    },
    /// The exit output can be claimed.
    Claimable {
        tip_height: u32,
        claimable_since: BlockRef,
        last_scanned_block: Option<BlockRef>,
    },
    /// A claim transaction has been broadcast.
    ClaimInProgress {
        tip_height: u32,
        claimable_since: BlockRef,
        claim_txid: String,
    },
    /// Terminal: the exit output was claimed (or spent deeper in the tree).
    Claimed {
        tip_height: u32,
        txid: String,
        block: BlockRef,
    },
    /// Terminal: the VTXO was already spent offchain, so the exit cannot
    /// proceed.
    VtxoAlreadySpent { tip_height: u32 },
    /// Terminal: an output the exit chain needs was spent on chain by someone
    /// else, in practice the server sweeping a VTXO that expired mid-exit.
    /// `spent_inputs` are the exit transaction inputs that were taken. A
    /// refresh may still be possible.
    VtxoSwept {
        tip_height: u32,
        spent_inputs: Vec<OutPoint>,
    },
    /// Resumable: the user canceled the exit before its final transaction
    /// was broadcast. The VTXO stays spendable.
    Canceled { tip_height: u32 },
}

impl From<&bark::exit::ExitState> for ExitState {
    fn from(s: &bark::exit::ExitState) -> Self {
        use bark::exit::ExitState as B;
        match s {
            B::Start(v) => Self::Start { tip_height: v.tip_height.to_u32() },
            B::Processing(v) => Self::Processing {
                tip_height: v.tip_height.to_u32(),
                transactions: v.transactions.iter().map(Into::into).collect(),
            },
            B::AwaitingDelta(v) => Self::AwaitingDelta {
                tip_height: v.tip_height.to_u32(),
                confirmed_block: (&v.confirmed_block).into(),
                claimable_height: v.claimable_height.to_u32(),
            },
            B::Claimable(v) => Self::Claimable {
                tip_height: v.tip_height.to_u32(),
                claimable_since: (&v.claimable_since).into(),
                last_scanned_block: v.last_scanned_block.as_ref().map(Into::into),
            },
            B::ClaimInProgress(v) => Self::ClaimInProgress {
                tip_height: v.tip_height.to_u32(),
                claimable_since: (&v.claimable_since).into(),
                claim_txid: v.claim_txid.to_string(),
            },
            B::Claimed(v) => Self::Claimed {
                tip_height: v.tip_height.to_u32(),
                txid: v.txid.to_string(),
                block: (&v.block).into(),
            },
            B::VtxoAlreadySpent(v) => Self::VtxoAlreadySpent { tip_height: v.tip_height.to_u32() },
            B::VtxoSwept(v) => Self::VtxoSwept {
                tip_height: v.tip_height.to_u32(),
                spent_inputs: v.spent_inputs.iter().map(Into::into).collect(),
            },
            B::Canceled(v) => Self::Canceled { tip_height: v.tip_height.to_u32() },
        }
    }
}

/// Lightweight discriminator for [`ExitState`], mirroring
/// `bark::exit::ExitStateKind`.
///
/// Carries no payload — use it where only the variant matters (e.g. reporting
/// which state blocked a cancellation). Serde/TS tags match upstream's
/// kebab-case serialization.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum ExitStateKind {
    Start,
    Processing,
    AwaitingDelta,
    Claimable,
    ClaimInProgress,
    Claimed,
    VtxoAlreadySpent,
    VtxoSwept,
    Canceled,
}

impl From<bark::exit::ExitStateKind> for ExitStateKind {
    fn from(k: bark::exit::ExitStateKind) -> Self {
        use bark::exit::ExitStateKind as B;
        match k {
            B::Start => Self::Start,
            B::Processing => Self::Processing,
            B::AwaitingDelta => Self::AwaitingDelta,
            B::Claimable => Self::Claimable,
            B::ClaimInProgress => Self::ClaimInProgress,
            B::Claimed => Self::Claimed,
            B::VtxoAlreadySpent => Self::VtxoAlreadySpent,
            B::VtxoSwept => Self::VtxoSwept,
            B::Canceled => Self::Canceled,
        }
    }
}

/// Why a [`Wallet::cancel_exit`](crate::core::Wallet::cancel_exit) did not
/// cancel the exit.
///
/// These are the *expected* negative outcomes of asking to cancel, not faults:
/// each is a normal state the exit can legitimately be in. Genuine failures
/// (database, chain source unreachable) still surface as `Err`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "kebab-case")]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum ExitCancelFailure {
    /// No exit was ever started for this VTXO.
    NotExiting,
    /// The exit has progressed past its abortable window. `state` is the
    /// state that blocked the cancellation.
    TooLate { state: ExitStateKind },
    /// The final exit transaction is already in the mempool or a block, so
    /// the exit can no longer be called off.
    AlreadyBroadcast { txid: String },
}

/// Outcome of a cancellation request.
///
/// `canceled` is `true` when the exit is now canceled — including when it was
/// already canceled by an earlier call, since cancellation is idempotent.
/// When `false`, `reason` says why.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct ExitCancelResult {
    pub canceled: bool,
    /// Present exactly when `canceled` is `false`.
    pub reason: Option<ExitCancelFailure>,
}

impl ExitCancelResult {
    /// The exit is canceled.
    pub(crate) fn canceled() -> Self {
        Self { canceled: true, reason: None }
    }

    /// The exit was left as-is, for the given reason.
    pub(crate) fn refused(reason: ExitCancelFailure) -> Self {
        Self { canceled: false, reason: Some(reason) }
    }
}

/// A VTXO that is being unilaterally exited
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct ExitVtxo {
    pub vtxo_id: String,
    pub amount_sats: u64,
    pub state: ExitState,
    pub is_claimable: bool,
}

impl From<&BarkExitVtxo> for ExitVtxo {
    fn from(ev: &BarkExitVtxo) -> Self {
        Self {
            vtxo_id: ev.id().to_string(),
            amount_sats: ev.amount().to_sat(),
            state: ev.state().into(),
            is_claimable: ev.is_claimable(),
        }
    }
}

/// Status of an exit progression
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct ExitProgressStatus {
    pub vtxo_id: String,
    pub state: ExitState,
    pub error: Option<String>,
}

impl From<BarkExitProgressStatus> for ExitProgressStatus {
    fn from(eps: BarkExitProgressStatus) -> Self {
        Self {
            vtxo_id: eps.vtxo_id.to_string(),
            state: (&eps.state).into(),
            error: eps.error.map(|e| e.to_string()),
        }
    }
}

/// Claim transaction for exited funds
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct ExitClaimTransaction {
    pub psbt_base64: String,
    pub fee_sats: u64,
    /// The exits the PSBT actually spends. Link the claim transaction to
    /// these, not to the ids you asked for.
    pub vtxo_ids: Vec<String>,
    /// Requested ids that were left out because they were not claimable when
    /// the PSBT was built. Empty when draining all claimable exits.
    pub skipped_vtxo_ids: Vec<String>,
}

/// The CPFP child of an exit transaction.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct ExitChildTransaction {
    pub txid: String,
    pub tx_hex: String,
    /// Who created the child. Only a `Wallet` child was paid for by this
    /// wallet, which is what fee attribution needs.
    pub origin: ExitTxOrigin,
    /// Effective fee rate of the child and its unconfirmed ancestors, in
    /// sat/kwu. `None` until the mempool has been queried for it.
    pub fee_rate_sat_per_kwu: Option<u64>,
    /// Fee of the child plus the fees of its unconfirmed ancestors. `None`
    /// until the mempool has been queried for it.
    pub total_fee_sats: Option<u64>,
}

/// One transaction of an exit: the exit transaction itself and, once it was
/// fee-bumped, the child that confirms it. A unilateral exit is a chain of
/// these, root first.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct ExitTransactionPackage {
    pub exit_txid: String,
    pub exit_tx_hex: String,
    pub child: Option<ExitChildTransaction>,
}

impl From<bark::exit::ExitTransactionPackage> for ExitTransactionPackage {
    fn from(p: bark::exit::ExitTransactionPackage) -> Self {
        use bitcoin::consensus::encode::serialize_hex;
        Self {
            exit_txid: p.exit.txid.to_string(),
            exit_tx_hex: serialize_hex(&p.exit.tx),
            child: p.child.map(|c| ExitChildTransaction {
                txid: c.info.txid.to_string(),
                tx_hex: serialize_hex(&c.info.tx),
                origin: (&c.origin).into(),
                fee_rate_sat_per_kwu: c.fee_info.map(|f| f.fee_rate.to_sat_per_kwu()),
                total_fee_sats: c.fee_info.map(|f| f.total_fee.to_sat()),
            }),
        }
    }
}

/// Detailed status of an exit transaction
///
/// Returned for live and finished exits alike by `get_exit_status`,
/// `list_all_exits`, `list_live_exits` and `list_finished_exits`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct ExitTransactionStatus {
    pub vtxo_id: String,
    pub state: ExitState,
    pub history: Option<Vec<ExitState>>,
    /// Same as `transactions.len()`. Kept for callers that only want the count.
    pub transaction_count: u32,
    /// The exit transaction chain, root first. Empty unless the status was
    /// requested with `include_transactions`.
    pub transactions: Vec<ExitTransactionPackage>,
}

impl From<bark::exit::ExitTransactionStatus> for ExitTransactionStatus {
    fn from(ets: bark::exit::ExitTransactionStatus) -> Self {
        Self {
            vtxo_id: ets.vtxo_id.to_string(),
            state: (&ets.state).into(),
            history: ets
                .history
                .map(|h| h.iter().map(Into::into).collect()),
            transaction_count: ets.transactions.len() as u32,
            transactions: ets.transactions.into_iter().map(Into::into).collect(),
        }
    }
}

/// A transaction of an exit chain, mirroring `bark::exit::TransactionInfo`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct ExitTransactionInfo {
    pub txid: String,
    /// Consensus-encoded transaction, hex.
    pub tx_hex: String,
}

impl From<bark::exit::TransactionInfo> for ExitTransactionInfo {
    fn from(t: bark::exit::TransactionInfo) -> Self {
        Self {
            txid: t.txid.to_string(),
            tx_hex: bitcoin::consensus::encode::serialize_hex(&t.tx),
        }
    }
}

/// Fee a transaction commits together with its unconfirmed ancestors,
/// mirroring `bark::exit::FeeInfo`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct ExitFeeInfo {
    /// Effective fee rate, in sat/kwu (lossless; divide by 250 for sat/vB).
    pub fee_rate_sat_per_kwu: u64,
    /// Sum of the transaction's own fee and the fees of its unconfirmed
    /// ancestors.
    pub total_fee_sats: u64,
}

impl From<bark::exit::FeeInfo> for ExitFeeInfo {
    fn from(f: bark::exit::FeeInfo) -> Self {
        Self {
            fee_rate_sat_per_kwu: f.fee_rate.to_sat_per_kwu(),
            total_fee_sats: f.total_fee.to_sat(),
        }
    }
}

/// How much of a [`PresignedExit`]'s remaining broadcast work could be funded
/// and signed, mirroring `bark::exit::PresignedExitStatus`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "kebab-case", rename_all_fields = "camelCase")]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum PresignedExitStatus {
    /// Every package has a signed child. The exit's broadcast completes once
    /// wave `exited_in_wave` (an index into [`PresignedExits::waves`])
    /// confirms; `None` when the chain is already confirmed and there is
    /// nothing to broadcast. `fee_info` totals the children's fees over the
    /// combined weight of this exit's packages.
    FullySigned {
        exited_in_wave: Option<u32>,
        fee_info: ExitFeeInfo,
    },
    /// Confirmed onchain funds ran out mid-chain: wave `last_wave` holds this
    /// exit's last signed package and [`PresignedExit::unsigned`] continues
    /// right after it.
    PartiallySigned { last_wave: u32 },
    /// No package could be funded.
    Unsigned,
}

impl From<bark::exit::PresignedExitStatus> for PresignedExitStatus {
    fn from(s: bark::exit::PresignedExitStatus) -> Self {
        use bark::exit::PresignedExitStatus as B;
        match s {
            B::FullySigned { exited_in_wave, fee_info } => Self::FullySigned {
                exited_in_wave: exited_in_wave.map(wave_index),
                fee_info: fee_info.into(),
            },
            B::PartiallySigned { last_wave } => Self::PartiallySigned {
                last_wave: wave_index(last_wave),
            },
            B::Unsigned => Self::Unsigned,
        }
    }
}

/// Wave indices are bounded by the number of exit transactions a wallet can
/// hold, so they always fit; saturate rather than panic all the same.
fn wave_index(i: usize) -> u32 {
    u32::try_from(i).unwrap_or(u32::MAX)
}

/// A signed CPFP child and the fee it commits, mirroring
/// `bark::exit::PresignedExitChildInfo`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PresignedExitChild {
    pub txid: String,
    /// Consensus-encoded signed transaction, hex.
    pub tx_hex: String,
    /// The exit transaction is zero-fee, so this is the fee of the whole
    /// package.
    pub fee_info: ExitFeeInfo,
}

/// An `(exit, child)` transaction package, broadcast as one unit (e.g. with
/// `submitpackage`), mirroring `bark::exit::PresignedExitPackage`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PresignedExitPackage {
    /// The VTXO whose exit this package advances.
    pub vtxo_id: String,
    /// The zero-fee exit transaction.
    pub exit: ExitTransactionInfo,
    /// The signed CPFP child paying for `exit`.
    pub child: PresignedExitChild,
}

impl From<bark::exit::PresignedExitPackage> for PresignedExitPackage {
    fn from(p: bark::exit::PresignedExitPackage) -> Self {
        Self {
            vtxo_id: p.vtxo_id.to_string(),
            exit: p.exit.into(),
            child: PresignedExitChild {
                txid: p.child.txid.to_string(),
                tx_hex: bitcoin::consensus::encode::serialize_hex(&p.child.tx),
                fee_info: p.child.fee_info.into(),
            },
        }
    }
}

/// One step of a presigned broadcast plan, mirroring
/// `bark::exit::PresignedWave`. See [`PresignedExits`] for the rules.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PresignedWave {
    /// Packages funded independently of each other, broadcast together.
    pub packages: Vec<PresignedExitPackage>,
    /// Exit transactions already in the mempool; they confirm with this wave.
    /// Only the first wave has any.
    pub in_mempool: Vec<ExitTransactionInfo>,
}

impl From<bark::exit::PresignedWave> for PresignedWave {
    fn from(w: bark::exit::PresignedWave) -> Self {
        Self {
            packages: w.packages.into_iter().map(Into::into).collect(),
            in_mempool: w.in_mempool.into_iter().map(Into::into).collect(),
        }
    }
}

/// The presigned broadcast work for one VTXO, mirroring
/// `bark::exit::PresignedExit`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PresignedExit {
    pub vtxo_id: String,
    pub status: PresignedExitStatus,
    /// The remaining tail of this exit's chain that could not be funded, in
    /// broadcast order; its first transaction spends this exit's last signed
    /// one. Empty when fully signed.
    pub unsigned: Vec<ExitTransactionInfo>,
}

impl From<bark::exit::PresignedExit> for PresignedExit {
    fn from(e: bark::exit::PresignedExit) -> Self {
        Self {
            vtxo_id: e.vtxo_id.to_string(),
            status: e.status.into(),
            unsigned: e.unsigned.into_iter().map(Into::into).collect(),
        }
    }
}

/// Presigned emergency-exit chains for a set of VTXOs: a wave-by-wave
/// broadcast plan plus a per-VTXO summary, mirroring
/// `bark::exit::PresignedExits`.
///
/// Execute the plan in wave order: broadcast all of a wave's packages
/// together, then wait for every one, and its `in_mempool` transactions, to
/// confirm before the next wave. Each wave's children spend change confirmed
/// by the previous wave.
///
/// Presigning reserves no funds, so a later spend from the onchain wallet can
/// invalidate the remaining children. It does reveal the change addresses
/// those children pay to. The children are fully signed: treat the result as
/// sensitive and store it accordingly.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PresignedExits {
    /// One entry per requested VTXO, in request order (duplicates resolve to
    /// their first occurrence).
    pub exits: Vec<PresignedExit>,
    /// The broadcast plan. Confirmed transactions are omitted, relayed ones are
    /// listed in [`PresignedWave::in_mempool`], and a package shared by several
    /// exits appears once, under the first exit that references it.
    pub waves: Vec<PresignedWave>,
}

impl From<bark::exit::PresignedExits> for PresignedExits {
    fn from(p: bark::exit::PresignedExits) -> Self {
        Self {
            exits: p.exits.into_iter().map(Into::into).collect(),
            waves: p.waves.into_iter().map(Into::into).collect(),
        }
    }
}

/// The exit transactions one VTXO still needs confirmed to be exited.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct RemainingExit {
    pub vtxo_id: String,
    /// Txids in broadcast order. Each one's transaction is in
    /// [`RemainingExitTransactions::transactions`]. Empty when the chain is
    /// already confirmed.
    pub txids: Vec<String>,
}

/// The exit transactions of a set of VTXOs still awaiting confirmation,
/// mirroring `bark::exit::RemainingExitTransactions`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct RemainingExitTransactions {
    /// One entry per requested VTXO, in request order (duplicates resolve to
    /// their first occurrence). Upstream returns a map; it is a list here so
    /// the order is stable across bindings.
    pub exits: Vec<RemainingExit>,
    /// Every transaction referenced by `exits` exactly once, parents before
    /// children.
    pub transactions: Vec<ExitTransactionInfo>,
}

impl RemainingExitTransactions {
    /// Converts bark's result, listing `exits` in the order of `requested`.
    /// Ids missing from `requested` (none, in practice) follow, sorted, so
    /// nothing upstream returned is dropped.
    pub(crate) fn from_bark(
        r: bark::exit::RemainingExitTransactions,
        requested: &[ark::VtxoId],
    ) -> Self {
        let mut map = r.exits;
        let mut ordered = Vec::with_capacity(map.len());
        for id in requested {
            if let Some(txids) = map.remove(id) {
                ordered.push((*id, txids));
            }
        }
        let mut rest: Vec<_> = map.into_iter().collect();
        rest.sort_unstable_by_key(|(id, _)| *id);
        ordered.extend(rest);

        Self {
            exits: ordered
                .into_iter()
                .map(|(id, txids)| RemainingExit {
                    vtxo_id: id.to_string(),
                    txids: txids.iter().map(|t| t.to_string()).collect(),
                })
                .collect(),
            transactions: r.transactions.into_iter().map(Into::into).collect(),
        }
    }
}

// ============================================================================
// Round Types
// ============================================================================

use bark::persist::models::{StoredRoundState, Unlocked};
use bark::round::RoundFlowKind as BarkRoundFlowKind;

/// Lifecycle phase of a round participation, mirroring
/// `bark::round::RoundFlowKind`.
///
/// Serde/TS tags match upstream's `Display` form: `"delegated-pending"` |
/// `"pending"` | `"ongoing"` | `"awaiting-confirmations"` | `"failed"` |
/// `"canceled"`.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum RoundFlowKind {
    /// Delegated participation waiting for its scheduled round. See
    /// [`RoundState::scheduled_height`].
    DelegatedPending,
    /// Interactive participation waiting for its round.
    Pending,
    /// The interactive part is being played out with the server.
    Ongoing,
    /// The round finished; waiting for its funding tx to confirm.
    AwaitingConfirmations,
    /// The participation failed.
    Failed,
    /// The user canceled the participation.
    Canceled,
}

impl From<BarkRoundFlowKind> for RoundFlowKind {
    fn from(k: BarkRoundFlowKind) -> Self {
        match k {
            BarkRoundFlowKind::DelegatedPending => Self::DelegatedPending,
            BarkRoundFlowKind::Pending => Self::Pending,
            BarkRoundFlowKind::Ongoing => Self::Ongoing,
            BarkRoundFlowKind::AwaitingConfirmations => Self::AwaitingConfirmations,
            BarkRoundFlowKind::Failed => Self::Failed,
            BarkRoundFlowKind::Canceled => Self::Canceled,
        }
    }
}

/// A pending round state
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct RoundState {
    pub id: u32,
    /// Whether the interactive part of the round is ongoing. Equivalent to
    /// `state` being `Pending` or `Ongoing`; kept for compatibility.
    pub ongoing: bool,
    /// Lifecycle phase of the participation.
    pub state: RoundFlowKind,
    /// Block height a delegated participation waits for, if it asked the
    /// server to schedule one. Only set while `state` is `DelegatedPending`.
    #[cfg_attr(feature = "wasm-web", tsify(optional))]
    pub scheduled_height: Option<u32>,
}

impl From<StoredRoundState<Unlocked>> for RoundState {
    fn from(rs: StoredRoundState<Unlocked>) -> Self {
        Self {
            id: rs.id().0,
            ongoing: rs.state().ongoing_participation(),
            state: rs.state().flow_kind().into(),
            scheduled_height: rs.state().scheduled_height().map(|h| h.to_u32()),
        }
    }
}

// ============================================================================
// Recovery Types
// ============================================================================

/// One bucket of a [`RecoveryReport`].
///
/// `total_sats` only sums the VTXOs whose amount is known, so it can
/// under-count `failed`, `abandoned` and `not_found`, where a VTXO may not
/// have been fetched. `vtxo_ids` is sorted as strings.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct RecoveryBucket {
    pub vtxo_ids: Vec<String>,
    pub total_sats: u64,
}

/// Arguments for `Wallet::import_vtxo` / `Wallet::import_vtxos`, mirroring
/// `bark::ImportVtxoArgs`.
///
/// Every field has a default, so callers only set what they need.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct ImportVtxoArgs {
    /// Gap limit for the key scan that decides whether we own the VTXOs,
    /// overriding `Config.vtxo_key_gap_limit` for this call.
    ///
    /// Default: none (use the wallet's configured limit)
    #[cfg_attr(feature = "wasm-web", tsify(optional))]
    #[cfg_attr(feature = "uniffi", uniffi(default = None))]
    #[serde(default)]
    pub gap_limit: Option<u32>,

    /// Import as spendable without asking the server for each VTXO's state.
    /// Skipping the check is faster but can leave the wallet holding spent
    /// VTXOs marked spendable, which then fail when selected as inputs.
    ///
    /// Default: false
    #[cfg_attr(feature = "wasm-web", tsify(optional))]
    #[cfg_attr(feature = "uniffi", uniffi(default = false))]
    #[serde(default)]
    pub skip_status_check: bool,

    /// Keep the VTXOs that import successfully even when another one in the
    /// batch fails; the returned ids are the ones that were kept.
    ///
    /// Default: false, so a single failure discards the whole batch.
    #[cfg_attr(feature = "wasm-web", tsify(optional))]
    #[cfg_attr(feature = "uniffi", uniffi(default = false))]
    #[serde(default)]
    pub allow_partial: bool,
}

impl From<ImportVtxoArgs> for bark::ImportVtxoArgs {
    fn from(a: ImportVtxoArgs) -> Self {
        bark::ImportVtxoArgs {
            gap_limit: a.gap_limit,
            skip_status_check: a.skip_status_check,
            allow_partial: a.allow_partial,
        }
    }
}

/// Outcome of a recovery scan: every VTXO id the scan looked at, bucketed by
/// what was decided about it.
///
/// `skipped` vs `failed` is the load-bearing distinction: a `skipped` VTXO was
/// decided not to be spendable (spent, exited, or reported non-spendable),
/// while a `failed` one could not be decided because of an error, so its funds
/// may still be missing. `failed` is retried by every sync while a seed
/// recovery is in progress, and via `recover_vtxos`; an `abandoned` one was
/// given up on after too many attempts; a `foreign` id instead sits beyond the
/// key-derivation gap limit and needs a wider scan.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct RecoveryReport {
    /// Spendable VTXOs that were successfully re-imported.
    pub recovered: RecoveryBucket,
    /// Deliberately left out: spent into a newer recovered VTXO, exited
    /// on-chain, or reported non-spendable by the server.
    pub skipped: RecoveryBucket,
    /// No matching key could be derived within the gap limit (the configured
    /// run of consecutive unused key indices, 250 by default). For a mailbox scan these are most likely this wallet's
    /// own VTXOs, keyed beyond the limit, so funds may be missing; retrying
    /// won't help, only a wider gap limit (see `Config.vtxo_key_gap_limit` and
    /// the `gap_limit` override on `recover_vtxos`). For `recover_vtxos` it
    /// just means the caller passed an id this wallet doesn't own.
    pub foreign: RecoveryBucket,
    /// Could not be decided due to an error. Not known to be spent, so funds
    /// may be missing. Retryable.
    pub failed: RecoveryBucket,
    /// Already fully exited on-chain.
    pub exited: RecoveryBucket,
    /// Reported unknown by the server. Recovery trusts the server, so these
    /// are not retried.
    pub not_found: RecoveryBucket,
    /// Given up on after `MAX_RECOVERY_ATTEMPTS` (40) attempts the server or
    /// the chain answered without deciding. Funds may be missing. One with a
    /// stored body stays locked, as it may be spent, and a later
    /// `recover_vtxos` tries it again.
    pub abandoned: RecoveryBucket,
    /// Whether the scan accounted for every VTXO: no `failed`, no `abandoned`,
    /// no `foreign`.
    pub is_complete: bool,
}

impl From<&bark::RecoveryReportEntry> for RecoveryBucket {
    fn from(e: &bark::RecoveryReportEntry) -> Self {
        // Upstream orders by `VtxoId`, whose byte order differs from the
        // displayed string; sort the strings for a stable, readable order.
        let mut vtxo_ids = e.ids().map(|id| id.to_string()).collect::<Vec<_>>();
        vtxo_ids.sort_unstable();
        Self { vtxo_ids, total_sats: e.total_amount().to_sat() }
    }
}

impl From<&bark::RecoveryReport> for RecoveryReport {
    fn from(r: &bark::RecoveryReport) -> Self {
        Self {
            recovered: r.recovered().into(),
            skipped: r.skipped().into(),
            foreign: r.foreign().into(),
            failed: r.failed().into(),
            exited: r.exited().into(),
            not_found: r.not_found().into(),
            abandoned: r.abandoned().into(),
            is_complete: r.is_complete(),
        }
    }
}

/// Outcome of the seed recovery as `Wallet::open` saw it, mirroring
/// `bark::RecoveryStatus`.
///
/// The variants are distinguishable on purpose: `NotRun` means nothing was
/// attempted (wallet already existed locally without a recovery in progress,
/// or `skip_recovery` was set), `Failed` means the recovery could not be
/// started or read back so funds may be missing, `InProgress` means it stopped
/// before it was done and resumes on every sync, and `Completed` carries the
/// final report — whose `is_complete` can still be `false` for individual
/// VTXOs.
///
/// Serde/TS tags: `"not-run"` | `"failed"` | `"in-progress"` | `"completed"`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum RecoveryStatus {
    /// No scan was attempted.
    NotRun,
    /// The recovery could not be started or its state could not be read
    /// back.
    Failed { message: String },
    /// The recovery stopped before it was done, on an error or on VTXOs it
    /// could not decide on yet. The report holds what it decided so far; its
    /// `failed` VTXOs are kept locked. Every sync resumes the recovery, see
    /// `Wallet::pending_recovery`.
    InProgress { report: RecoveryReport },
    /// The recovery is done: no VTXO is left failed.
    Completed { report: RecoveryReport },
}

impl RecoveryStatus {
    /// The report, if the recovery got far enough to produce one.
    pub fn report(&self) -> Option<&RecoveryReport> {
        match self {
            Self::InProgress { report } | Self::Completed { report } => Some(report),
            Self::NotRun | Self::Failed { .. } => None,
        }
    }
}

impl From<bark::RecoveryStatus> for RecoveryStatus {
    fn from(s: bark::RecoveryStatus) -> Self {
        match s {
            bark::RecoveryStatus::NotRun => Self::NotRun,
            // Alternate form joins the anyhow cause chain on one line.
            bark::RecoveryStatus::Failed(e) => Self::Failed { message: format!("{:#}", e) },
            bark::RecoveryStatus::InProgress(r) => Self::InProgress { report: (&r).into() },
            bark::RecoveryStatus::Completed(r) => Self::Completed { report: (&r).into() },
        }
    }
}

// ============================================================================
// Callback Wallet Types
// ============================================================================

/// A Bitcoin transaction output destination
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct Destination {
    pub address: String,
    pub amount_sats: u64,
}

/// A Bitcoin transaction outpoint (reference to a previous output)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct OutPoint {
    pub txid: String,
    pub vout: u32,
}

impl From<&bitcoin::OutPoint> for OutPoint {
    fn from(o: &bitcoin::OutPoint) -> Self {
        Self {
            txid: o.txid.to_string(),
            vout: o.vout,
        }
    }
}

/// Reference to a block in the blockchain
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct BlockRef {
    pub height: u32,
    pub hash: String,
}

impl From<&bark_bitcoin_ext::BlockRef> for BlockRef {
    fn from(br: &bark_bitcoin_ext::BlockRef) -> Self {
        Self {
            height: br.height.to_u32(),
            hash: br.hash.to_string(),
        }
    }
}

/// Parameters for creating a CPFP (Child Pays For Parent) transaction
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct CpfpParams {
    /// Hex-encoded zero-fee parent transaction to bump through its P2A anchor
    pub tx_hex: String,
    /// `"Effective"` for a fresh child, `"Rbf"` to replace a child already in
    /// the mempool. The `Rbf`-only fields below are `None` for `"Effective"`.
    pub fees_type: String,
    /// Effective fee rate the whole package (parent + child) must reach, in
    /// sat/vB rounded up
    pub effective_fee_rate_sat_per_vb: u64,
    /// `Rbf` only: total fee of the transactions the new child replaces
    pub replaced_fee_sats: Option<u64>,
    /// `Rbf` only: fee rate of the transactions the new child replaces, in
    /// sat/vB rounded up
    pub replaced_fee_rate_sat_per_vb: Option<u64>,
    /// `Rbf` only: the node's incremental relay fee, in sat/vB rounded up
    pub incremental_relay_fee_sat_per_vb: Option<u64>,
    /// `Rbf` only: whether the parent is already in the mempool. If not, the
    /// parent is part of the replacement.
    pub parent_in_mempool: Option<bool>,
}

// ============================================================================
// Payment requests
// ============================================================================

/// A payment destination, mirroring `bark::movement::PaymentMethod`.
///
/// Each variant carries the destination as a string. `type` and `value`
/// match upstream's JSON encoding (`{"type":"ark","value":"tark1…"}`), which
/// is what `Movement.sent_to_addresses` and `Movement.received_on_addresses`
/// hold and what `Wallet::history_by_payment_method` takes.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "kebab-case")]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum PaymentMethod {
    /// An Ark address.
    Ark { value: String },
    /// An onchain bitcoin address.
    Bitcoin { value: String },
    /// A hex-encoded onchain output script. bark cannot pay these yet.
    OutputScript { value: String },
    /// A lightning invoice, BOLT11 or BOLT12. `decode_bolt11_invoice` reads
    /// a BOLT11 invoice's amount, description, payee and expiry.
    Invoice { value: String },
    /// A reusable BOLT12 offer.
    Offer { value: String },
    /// A lightning address (`user@domain`), resolved over LNURL-pay.
    LightningAddress { value: String },
    /// An LNURL-pay link (`lnurl1…`).
    Lnurl { value: String },
    /// A destination bark recognizes but cannot pay, such as an Arkade
    /// address.
    Custom { value: String },
}

impl PaymentMethod {
    /// Upstream's type tag and value string.
    fn type_value(&self) -> (&'static str, &str) {
        match self {
            Self::Ark { value } => ("ark", value),
            Self::Bitcoin { value } => ("bitcoin", value),
            Self::OutputScript { value } => ("output-script", value),
            Self::Invoice { value } => ("invoice", value),
            Self::Offer { value } => ("offer", value),
            Self::LightningAddress { value } => ("lightning-address", value),
            Self::Lnurl { value } => ("lnurl", value),
            Self::Custom { value } => ("custom", value),
        }
    }

    /// Parse back into bark's type. Fails if `value` is not a valid
    /// destination of its kind.
    pub(crate) fn to_bark(&self) -> anyhow::Result<bark::movement::PaymentMethod> {
        let (type_str, value) = self.type_value();
        bark::movement::PaymentMethod::from_type_value(type_str, value)
    }
}

impl From<&bark::movement::PaymentMethod> for PaymentMethod {
    fn from(m: &bark::movement::PaymentMethod) -> Self {
        use bark::movement::PaymentMethod as B;

        let value = m.value_string();
        match m {
            B::Ark(_) => Self::Ark { value },
            B::Bitcoin(_) => Self::Bitcoin { value },
            B::OutputScript(_) => Self::OutputScript { value },
            B::Invoice(_) => Self::Invoice { value },
            B::Offer(_) => Self::Offer { value },
            B::LightningAddress(_) => Self::LightningAddress { value },
            B::Lnurl(_) => Self::Lnurl { value },
            B::Custom(_) => Self::Custom { value },
        }
    }
}

/// Why an option of a [`PaymentRequest`] may not be payable, mirroring
/// `bark::PaymentMethodParsingError`.
///
/// Returned as data in [`AvailablePaymentMethod::errors`] rather than as an
/// error, so the caller can still show every option and flag the bad ones.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "kebab-case", rename_all_fields = "camelCase")]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum PaymentMethodParsingError {
    /// The destination is for another bitcoin network than the wallet's.
    NetworkMismatch,
    /// This wallet cannot pay the Ark address, for example because it
    /// belongs to another Ark server. `reason` says which check failed.
    InvalidArkAddress { reason: String },
    /// The invoice or offer asks for `expected_sats`, but the URI's `amount`
    /// is `got_sats`.
    AmountMismatch { expected_sats: u64, got_sats: u64 },
    /// The destination's amount is invalid.
    InvalidAmount,
    /// bark recognizes the destination but cannot pay it.
    Unsupported,
}

impl From<&bark::PaymentMethodParsingError> for PaymentMethodParsingError {
    fn from(e: &bark::PaymentMethodParsingError) -> Self {
        use bark::PaymentMethodParsingError as B;
        match e {
            B::NetworkMismatch => Self::NetworkMismatch,
            B::InvalidArkAddress(e) => Self::InvalidArkAddress { reason: e.to_string() },
            B::AmountMismatch { expected, got } => Self::AmountMismatch {
                expected_sats: expected.to_sat(),
                got_sats: got.to_sat(),
            },
            B::InvalidAmount => Self::InvalidAmount,
            B::Unsupported => Self::Unsupported,
        }
    }
}

/// Fields of a BOLT11 invoice, as returned by `decode_bolt11_invoice`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct Bolt11InvoiceDetails {
    pub payment_hash: String,
    /// `None` for an amountless invoice, which needs an amount to pay.
    pub amount_msat: Option<u64>,
    /// `amount_msat` rounded up to whole sats.
    pub amount_sats: Option<u64>,
    /// The description, when the invoice carries it directly.
    pub description: Option<String>,
    /// The hex SHA-256 of the description, when the invoice carries that
    /// instead.
    pub description_hash: Option<String>,
    /// The payee's node public key, hex.
    pub payee_pubkey: String,
    /// Creation time, in unix seconds.
    pub created_at: u64,
    /// Expiry time, in unix seconds. Compare it with the current time to
    /// tell whether the invoice can still be paid.
    pub expires_at: u64,
}

impl From<&ark::lightning::Bolt11Invoice> for Bolt11InvoiceDetails {
    fn from(invoice: &ark::lightning::Bolt11Invoice) -> Self {
        use lightning_invoice::Bolt11InvoiceDescriptionRef as Description;

        let (description, description_hash) = match invoice.description() {
            Description::Direct(d) => (Some(d.to_string()), None),
            Description::Hash(h) => (None, Some(h.0.to_string())),
        };
        let amount_msat = invoice.amount_milli_satoshis();
        let created_at = invoice.duration_since_epoch().as_secs();

        Self {
            payment_hash: ark::lightning::PaymentHash::from(invoice).to_string(),
            amount_msat,
            amount_sats: amount_msat.map(|m| bitcoin::Amount::from_msat_ceil(m).to_sat()),
            description,
            description_hash,
            payee_pubkey: invoice.get_payee_pub_key().to_string(),
            created_at,
            expires_at: created_at.saturating_add(invoice.expiry_time().as_secs()),
        }
    }
}

/// One way to pay a [`PaymentRequest`], mirroring
/// `bark::AvailablePaymentMethod`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct AvailablePaymentMethod {
    pub method: PaymentMethod,
    /// Why this option may not be payable. Empty when bark found no problem.
    pub errors: Vec<PaymentMethodParsingError>,
    /// Whether `Wallet::send_payment` needs an amount for this method: always
    /// for addresses and LNURL, and for invoices and offers that carry none.
    pub requires_amount: bool,
}

impl From<&bark::AvailablePaymentMethod> for AvailablePaymentMethod {
    fn from(o: &bark::AvailablePaymentMethod) -> Self {
        Self {
            method: (&o.method).into(),
            errors: o.errors.iter().map(Into::into).collect(),
            requires_amount: o.method.requires_amount(),
        }
    }
}

/// A parsed payment string, mirroring `bark::PaymentRequest`.
///
/// `label` and `message` come from a BIP 321 URI. For a bare BOLT11 invoice
/// or BOLT12 offer, `amount_sats` is its amount and `message` its
/// description.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct PaymentRequest {
    pub amount_sats: Option<u64>,
    pub label: Option<String>,
    pub message: Option<String>,
    /// Every destination found, in the order bark parsed them.
    pub options: Vec<AvailablePaymentMethod>,
    /// The option to pay when the user doesn't pick one: Ark, then
    /// lightning, then onchain, skipping options with errors. `None` when
    /// every option has errors.
    pub default_option: Option<AvailablePaymentMethod>,
}

impl From<bark::PaymentRequest> for PaymentRequest {
    fn from(r: bark::PaymentRequest) -> Self {
        let default_option = r.default_option().map(Into::into);
        Self {
            amount_sats: r.amount.map(|a| a.to_sat()),
            options: r.options.iter().map(Into::into).collect(),
            label: r.label,
            message: r.message,
            default_option,
        }
    }
}

/// Outcome of `Wallet::send_payment`, mirroring `bark::PaymentInitOutput`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "kebab-case", rename_all_fields = "camelCase")]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum PaymentResult {
    /// The onchain payment's transaction was broadcast.
    Onchain { txid: String },
    /// The lightning payment was started. Unless `wait` was set it may still
    /// be in flight: follow it with `lightning_send_state(payment_hash)`.
    /// `invoice` is the one paid, which for an offer, lightning address or
    /// LNURL is the one bark fetched.
    Lightning { invoice: String, payment_hash: String },
    /// The Ark payment was made. `movement_id` is the movement that records
    /// it.
    Ark { movement_id: u32 },
}

impl From<bark::PaymentInitOutput> for PaymentResult {
    fn from(o: bark::PaymentInitOutput) -> Self {
        match o {
            bark::PaymentInitOutput::Onchain(txid) => Self::Onchain { txid: txid.to_string() },
            bark::PaymentInitOutput::Lightning(invoice) => Self::Lightning {
                payment_hash: invoice.payment_hash().to_string(),
                invoice: invoice.to_string(),
            },
            bark::PaymentInitOutput::Ark(movement_id) => Self::Ark { movement_id: movement_id.0 },
        }
    }
}

fn default_true() -> bool {
    true
}

/// What `Wallet::bip321_uri` puts in the URI. Every field is optional.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct Bip321UriOptions {
    /// The amount to request. A lightning invoice needs one.
    ///
    /// Default: none
    #[cfg_attr(feature = "uniffi", uniffi(default = None))]
    #[cfg_attr(feature = "wasm-web", tsify(optional))]
    pub amount_sats: Option<u64>,

    /// Default: none
    #[cfg_attr(feature = "uniffi", uniffi(default = None))]
    #[cfg_attr(feature = "wasm-web", tsify(optional))]
    pub label: Option<String>,

    /// Default: none
    #[cfg_attr(feature = "uniffi", uniffi(default = None))]
    #[cfg_attr(feature = "wasm-web", tsify(optional))]
    pub message: Option<String>,

    /// Whether to include a new Ark address.
    ///
    /// Default: true
    #[cfg_attr(feature = "uniffi", uniffi(default = true))]
    #[cfg_attr(feature = "wasm-web", tsify(optional))]
    #[serde(default = "default_true")]
    pub ark: bool,

    /// Whether to include a new BOLT11 invoice. Ignored without
    /// `amount_sats`.
    ///
    /// Default: true
    #[cfg_attr(feature = "uniffi", uniffi(default = true))]
    #[cfg_attr(feature = "wasm-web", tsify(optional))]
    #[serde(default = "default_true")]
    pub lightning: bool,

    /// An onchain address to include, such as one from the host's own
    /// onchain wallet. It must be for the wallet's network. As in bark's own
    /// builder, a mainnet address goes in the URI path and any other in a
    /// `tb=` parameter.
    ///
    /// Default: none
    #[cfg_attr(feature = "uniffi", uniffi(default = None))]
    #[cfg_attr(feature = "wasm-web", tsify(optional))]
    pub onchain_address: Option<String>,
}

/// A BIP 321 URI built by `Wallet::bip321_uri`, with the destinations in it.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct Bip321Uri {
    /// The `bitcoin:` URI, for example to show as a QR code.
    pub uri: String,
    /// The new Ark address in the URI.
    pub ark_address: Option<String>,
    /// The new invoice in the URI. Follow the receive by its `payment_hash`.
    pub lightning_invoice: Option<LightningInvoice>,
    /// The onchain address in the URI.
    pub onchain_address: Option<String>,
}

impl From<&bark::payment_request::BarkBip321Uri> for Bip321Uri {
    fn from(uri: &bark::payment_request::BarkBip321Uri) -> Self {
        let onchain_address = uri.address().cloned()
            .or_else(|| uri.tb().first().map(|f| f.inner().clone()));
        Self {
            uri: uri.to_string(),
            ark_address: uri.extensions().ark().first().map(|f| f.inner().to_string()),
            lightning_invoice: uri.lightning().first().map(|f| {
                let invoice = f.inner();
                LightningInvoice {
                    invoice: invoice.to_string(),
                    payment_hash: ark::lightning::PaymentHash::from(invoice).to_string(),
                    amount_sats: invoice.amount_milli_satoshis()
                        .map_or(0, |m| bitcoin::Amount::from_msat_ceil(m).to_sat()),
                }
            }),
            onchain_address: onchain_address.map(|a| a.to_string()),
        }
    }
}

// ============================================================================
// WalletNotification
// ============================================================================

/// A notification event from the wallet
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(tag = "type")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum WalletNotification {
    /// A new movement was created
    MovementCreated { movement: Movement },
    /// An existing movement was updated
    MovementUpdated { movement: Movement },
    /// The notification channel is lagging (notifications were dropped)
    ChannelLagging,
}

impl From<bark::WalletNotification> for WalletNotification {
    fn from(n: bark::WalletNotification) -> Self {
        match n {
            bark::WalletNotification::MovementCreated { movement } => {
                WalletNotification::MovementCreated {
                    movement: movement.into(),
                }
            }
            bark::WalletNotification::MovementUpdated { movement } => {
                WalletNotification::MovementUpdated {
                    movement: movement.into(),
                }
            }
            bark::WalletNotification::ChannelLagging => WalletNotification::ChannelLagging,
        }
    }
}

// ============================================================================
// Nostr Wallet Connect
// ============================================================================

/// A NIP-47 method an NWC connection can be granted.
// Serde/TS names are the NIP-47 method names, e.g. `"pay_invoice"`.
#[cfg(feature = "nwc")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "wasm-web", derive(tsify::Tsify))]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum NwcMethod {
    /// Read the wallet's network, block height and supported methods.
    GetInfo,
    /// Read the wallet's spendable balance.
    GetBalance,
    /// Create Lightning invoices that pay into the wallet.
    MakeInvoice,
    /// Look up an invoice or payment of the wallet.
    LookupInvoice,
    /// Pay Lightning invoices from the wallet. There is no spending limit:
    /// an app granted this can spend the whole balance.
    PayInvoice,
    /// Read the wallet's Lightning payment history.
    ListTransactions,
}

#[cfg(feature = "nwc")]
impl NwcMethod {
    pub const ALL: [NwcMethod; 6] = [
        NwcMethod::GetInfo,
        NwcMethod::GetBalance,
        NwcMethod::MakeInvoice,
        NwcMethod::LookupInvoice,
        NwcMethod::PayInvoice,
        NwcMethod::ListTransactions,
    ];

    /// The NIP-47 method name, e.g. `"pay_invoice"`, as the connection
    /// store records it.
    pub fn as_str(self) -> &'static str {
        match self {
            NwcMethod::GetInfo => "get_info",
            NwcMethod::GetBalance => "get_balance",
            NwcMethod::MakeInvoice => "make_invoice",
            NwcMethod::LookupInvoice => "lookup_invoice",
            NwcMethod::PayInvoice => "pay_invoice",
            NwcMethod::ListTransactions => "list_transactions",
        }
    }

    /// The method a NIP-47 method name names, if it is one bark serves.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|m| m.as_str() == name)
    }
}

/// An app's NWC access to the wallet.
#[cfg(feature = "nwc")]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct NwcConnection {
    /// The name the connection was created with.
    pub name: String,
    /// The app's public key, hex-encoded. Identifies the connection, e.g. to
    /// revoke it.
    pub client_pubkey: String,
    /// The methods the app may call.
    pub methods: Vec<NwcMethod>,
    /// RFC 3339.
    pub created_at: String,
}

/// A newly created NWC connection.
#[cfg(feature = "nwc")]
#[derive(Clone, Serialize, Deserialize)]
#[cfg_attr(
    feature = "wasm-web",
    derive(tsify::Tsify),
    serde(rename_all = "camelCase")
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct NwcConnectionUri {
    /// The `nostr+walletconnect://` URI to hand to the app. It carries the
    /// app's secret key, which the wallet does not keep: whoever holds the
    /// URI can use the connection. Show it once, never log or store it.
    pub uri: String,
    /// The app's public key, hex-encoded, as [`NwcConnection`] lists it.
    pub client_pubkey: String,
}

/// Leaves the URI out: it carries the app's secret key.
#[cfg(feature = "nwc")]
impl std::fmt::Debug for NwcConnectionUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NwcConnectionUri")
            .field("uri", &"<redacted>")
            .field("client_pubkey", &self.client_pubkey)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_vtxo_args_default_to_upstream_defaults() {
        let args: bark::ImportVtxoArgs = ImportVtxoArgs::default().into();
        assert_eq!(args.gap_limit, None);
        assert!(!args.skip_status_check);
        assert!(!args.allow_partial);
    }

    #[test]
    fn import_vtxo_args_are_forwarded() {
        let args: bark::ImportVtxoArgs = ImportVtxoArgs {
            gap_limit: Some(1_000),
            skip_status_check: true,
            allow_partial: true,
        }.into();
        assert_eq!(args.gap_limit, Some(1_000));
        assert!(args.skip_status_check);
        assert!(args.allow_partial);
    }
    use std::str::FromStr;

    fn txid(byte: u8) -> bitcoin::Txid {
        bitcoin::Txid::from_str(&hex::encode([byte; 32])).unwrap()
    }

    fn block_ref(height: u32, byte: u8) -> bark_bitcoin_ext::BlockRef {
        bark_bitcoin_ext::BlockRef {
            height: height.into(),
            hash: bitcoin::BlockHash::from_str(&hex::encode([byte; 32])).unwrap(),
        }
    }

    // ------------------------------------------------------------------
    // VtxoState (item: locked-VTXO lock reasons)
    // ------------------------------------------------------------------

    #[test]
    fn vtxo_state_conversion_carries_lock_holder() {
        use bark::movement::MovementId;
        use bark::vtxo::{VtxoLockHolder as B, VtxoState as BS};

        let cases: Vec<(BS, VtxoState)> = vec![
            (BS::Spendable, VtxoState::Spendable),
            (BS::Spent, VtxoState::Spent),
            (BS::Exited, VtxoState::Exited),
            (
                BS::Locked { holder: None },
                VtxoState::Locked { holder: None },
            ),
            (
                BS::Locked { holder: Some(B::Movement { id: MovementId(7) }) },
                VtxoState::Locked {
                    holder: Some(VtxoLockHolder::Movement { id: 7 }),
                },
            ),
            (
                BS::Locked { holder: Some(B::Action { id: "ln-pay:abc".into() }) },
                VtxoState::Locked {
                    holder: Some(VtxoLockHolder::Action { id: "ln-pay:abc".into() }),
                },
            ),
        ];
        for (upstream, expected) in cases {
            assert_eq!(VtxoState::from(&upstream), expected);
        }
    }

    #[test]
    fn vtxo_lock_holder_roundtrips_through_upstream() {
        // The lock guard in `unlock_vtxos` compares a holder the caller passed
        // in against one read back from the database, so the two directions
        // must agree exactly or a caller could never release its own lock.
        let cases = vec![
            VtxoLockHolder::Action { id: "exit:abc123".into() },
            VtxoLockHolder::Movement { id: 42 },
        ];
        for holder in cases {
            let upstream = bark::vtxo::VtxoLockHolder::from(holder.clone());
            assert_eq!(VtxoLockHolder::from(&upstream), holder);
        }
    }

    #[test]
    fn exit_state_kind_conversion_covers_all_variants() {
        for kind in bark::exit::ExitStateKind::ALL {
            // Exercises every arm; a new upstream variant fails to compile
            // rather than silently mapping to something wrong.
            let _: ExitStateKind = (*kind).into();
        }
    }

    fn tx(lock_time: u32) -> bitcoin::Transaction {
        bitcoin::Transaction {
            version: bitcoin::transaction::Version::non_standard(3),
            lock_time: bitcoin::absolute::LockTime::from_consensus(lock_time),
            input: vec![],
            output: vec![bitcoin::TxOut {
                value: bitcoin::Amount::from_sat(330),
                script_pubkey: bitcoin::ScriptBuf::new(),
            }],
        }
    }

    fn tx_info(lock_time: u32) -> bark::exit::TransactionInfo {
        let tx = tx(lock_time);
        bark::exit::TransactionInfo { txid: tx.compute_txid(), tx }
    }

    fn vtxo_id(byte: u8) -> ark::VtxoId {
        ark::VtxoId::from_slice(&[byte; 36]).unwrap()
    }

    #[test]
    fn exit_fee_info_is_lossless() {
        let fee = bark::exit::FeeInfo {
            fee_rate: bitcoin::FeeRate::from_sat_per_kwu(253),
            total_fee: bitcoin::Amount::from_sat(1_234),
        };
        assert_eq!(
            ExitFeeInfo::from(fee),
            ExitFeeInfo { fee_rate_sat_per_kwu: 253, total_fee_sats: 1_234 },
        );
    }

    #[test]
    fn presigned_exits_conversion_carries_every_field() {
        use bark::exit as B;

        let fee = B::FeeInfo {
            fee_rate: bitcoin::FeeRate::from_sat_per_kwu(500),
            total_fee: bitcoin::Amount::from_sat(700),
        };
        let (exit, child, relayed, tail) = (tx_info(1), tx_info(2), tx_info(3), tx_info(4));
        let upstream = B::PresignedExits {
            exits: vec![
                B::PresignedExit {
                    vtxo_id: vtxo_id(1),
                    status: B::PresignedExitStatus::FullySigned { exited_in_wave: Some(0), fee_info: fee },
                    unsigned: vec![],
                },
                B::PresignedExit {
                    vtxo_id: vtxo_id(2),
                    status: B::PresignedExitStatus::PartiallySigned { last_wave: 3 },
                    unsigned: vec![tail.clone()],
                },
                B::PresignedExit {
                    vtxo_id: vtxo_id(3),
                    status: B::PresignedExitStatus::Unsigned,
                    unsigned: vec![],
                },
                B::PresignedExit {
                    vtxo_id: vtxo_id(4),
                    status: B::PresignedExitStatus::FullySigned { exited_in_wave: None, fee_info: fee },
                    unsigned: vec![],
                },
            ],
            waves: vec![B::PresignedWave {
                packages: vec![B::PresignedExitPackage {
                    vtxo_id: vtxo_id(1),
                    exit: exit.clone(),
                    child: B::PresignedExitChildInfo {
                        txid: child.txid,
                        tx: child.tx.clone(),
                        fee_info: fee,
                    },
                }],
                in_mempool: vec![relayed.clone()],
            }],
        };

        let ffi_fee = ExitFeeInfo { fee_rate_sat_per_kwu: 500, total_fee_sats: 700 };
        let hex = |t: &B::TransactionInfo| bitcoin::consensus::encode::serialize_hex(&t.tx);
        let info = |t: &B::TransactionInfo| ExitTransactionInfo {
            txid: t.txid.to_string(),
            tx_hex: hex(t),
        };

        let got = PresignedExits::from(upstream);
        assert_eq!(got.exits, vec![
            PresignedExit {
                vtxo_id: vtxo_id(1).to_string(),
                status: PresignedExitStatus::FullySigned { exited_in_wave: Some(0), fee_info: ffi_fee.clone() },
                unsigned: vec![],
            },
            PresignedExit {
                vtxo_id: vtxo_id(2).to_string(),
                status: PresignedExitStatus::PartiallySigned { last_wave: 3 },
                unsigned: vec![info(&tail)],
            },
            PresignedExit {
                vtxo_id: vtxo_id(3).to_string(),
                status: PresignedExitStatus::Unsigned,
                unsigned: vec![],
            },
            PresignedExit {
                vtxo_id: vtxo_id(4).to_string(),
                status: PresignedExitStatus::FullySigned { exited_in_wave: None, fee_info: ffi_fee.clone() },
                unsigned: vec![],
            },
        ]);
        assert_eq!(got.waves, vec![PresignedWave {
            packages: vec![PresignedExitPackage {
                vtxo_id: vtxo_id(1).to_string(),
                exit: info(&exit),
                child: PresignedExitChild {
                    txid: child.txid.to_string(),
                    tx_hex: hex(&child),
                    fee_info: ffi_fee,
                },
            }],
            in_mempool: vec![info(&relayed)],
        }]);

        // The hex is the consensus encoding, so it decodes back to the txid.
        let decoded: bitcoin::Transaction = bitcoin::consensus::encode::deserialize_hex(
            &got.waves[0].packages[0].child.tx_hex,
        ).unwrap();
        assert_eq!(decoded.compute_txid(), child.txid);
    }

    #[test]
    fn presigned_exit_status_serde_tags() {
        let fee = ExitFeeInfo { fee_rate_sat_per_kwu: 1, total_fee_sats: 2 };
        let v = serde_json::to_value(PresignedExitStatus::FullySigned {
            exited_in_wave: Some(1),
            fee_info: fee,
        }).unwrap();
        assert_eq!(v["type"], serde_json::json!("fully-signed"));
        assert_eq!(v["exitedInWave"], serde_json::json!(1));
        // Record fields are camelCased only for wasm-web, like every record.
        assert!(v["feeInfo"].is_object(), "{v}");

        let v = serde_json::to_value(PresignedExitStatus::PartiallySigned { last_wave: 2 }).unwrap();
        assert_eq!(v["type"], serde_json::json!("partially-signed"));
        assert_eq!(v["lastWave"], serde_json::json!(2));

        let v = serde_json::to_value(PresignedExitStatus::Unsigned).unwrap();
        assert_eq!(v["type"], serde_json::json!("unsigned"));
    }

    /// Upstream returns a map; the FFI list follows the request order,
    /// skipping duplicates, and keeps anything the request did not name.
    #[test]
    fn remaining_exit_transactions_follow_request_order() {
        let (a, b, c) = (tx_info(1), tx_info(2), tx_info(3));
        let upstream = bark::exit::RemainingExitTransactions {
            exits: [
                (vtxo_id(1), vec![a.txid]),
                (vtxo_id(2), vec![a.txid, b.txid]),
                (vtxo_id(3), vec![]),
                (vtxo_id(9), vec![c.txid]),
                (vtxo_id(8), vec![]),
            ].into_iter().collect(),
            transactions: vec![a.clone(), b.clone(), c.clone()],
        };

        let requested = [vtxo_id(3), vtxo_id(1), vtxo_id(3), vtxo_id(2)];
        let got = RemainingExitTransactions::from_bark(upstream, &requested);

        let order: Vec<_> = got.exits.iter().map(|e| e.vtxo_id.clone()).collect();
        assert_eq!(order, [3, 1, 2, 8, 9].map(|b| vtxo_id(b).to_string()));
        assert_eq!(got.exits[2].txids, vec![a.txid.to_string(), b.txid.to_string()]);
        assert!(got.exits[0].txids.is_empty());
        let txids: Vec<_> = got.transactions.iter().map(|t| t.txid.clone()).collect();
        assert_eq!(txids, [&a, &b, &c].map(|t| t.txid.to_string()));
    }

    #[test]
    fn exit_cancel_result_serde_shape() {
        let ok = serde_json::to_value(ExitCancelResult::canceled()).unwrap();
        assert_eq!(ok["canceled"], serde_json::json!(true));
        assert!(ok["reason"].is_null());

        let refused = ExitCancelResult::refused(ExitCancelFailure::TooLate {
            state: ExitStateKind::Claimed,
        });
        let v = serde_json::to_value(&refused).unwrap();
        assert_eq!(v["canceled"], serde_json::json!(false));
        assert_eq!(v["reason"]["type"], serde_json::json!("too-late"));
        assert_eq!(v["reason"]["state"], serde_json::json!("claimed"));

        let broadcast = ExitCancelResult::refused(ExitCancelFailure::AlreadyBroadcast {
            txid: "ff00".into(),
        });
        let v = serde_json::to_value(&broadcast).unwrap();
        assert_eq!(v["reason"]["type"], serde_json::json!("already-broadcast"));
        assert_eq!(v["reason"]["txid"], serde_json::json!("ff00"));
    }

    #[test]
    fn vtxo_state_serde_tags_match_upstream() {
        let locked = VtxoState::Locked {
            holder: Some(VtxoLockHolder::Movement { id: 42 }),
        };
        let json = serde_json::to_value(&locked).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "type": "locked",
                "holder": { "type": "movement", "id": 42 }
            })
        );
        assert_eq!(
            serde_json::to_value(&VtxoState::Spendable).unwrap(),
            serde_json::json!({ "type": "spendable" })
        );
    }

    // ------------------------------------------------------------------
    // ExitState (item: exit state block refs / txids)
    // ------------------------------------------------------------------

    #[test]
    fn exit_state_conversion_carries_payloads() {
        use bark::exit as be;

        let claimed: ExitState = (&be::ExitState::Claimed(be::ExitClaimedState {
            tip_height: 900.into(),
            txid: txid(0xaa),
            block: block_ref(890, 0xbb),
        }))
            .into();
        assert_eq!(
            claimed,
            ExitState::Claimed {
                tip_height: 900,
                txid: hex::encode([0xaa; 32]),
                block: BlockRef { height: 890, hash: hex::encode([0xbb; 32]) },
            }
        );

        let awaiting: ExitState = (&be::ExitState::AwaitingDelta(be::ExitAwaitingDeltaState {
            tip_height: 100.into(),
            confirmed_block: block_ref(95, 0xcc),
            claimable_height: 107.into(),
        }))
            .into();
        assert_eq!(
            awaiting,
            ExitState::AwaitingDelta {
                tip_height: 100,
                confirmed_block: BlockRef { height: 95, hash: hex::encode([0xcc; 32]) },
                claimable_height: 107,
            }
        );

        // bark does not re-export `ExitVtxoSweptState`, so build it from its
        // serde form.
        let upstream: be::ExitState = serde_json::from_value(serde_json::json!({
            "type": "vtxo-swept",
            "tip_height": 300,
            "spent_inputs": [format!("{}:2", txid(0xdd))],
        }))
        .unwrap();
        let swept: ExitState = (&upstream).into();
        assert_eq!(
            swept,
            ExitState::VtxoSwept {
                tip_height: 300,
                spent_inputs: vec![OutPoint { txid: hex::encode([0xdd; 32]), vout: 2 }],
            }
        );
    }

    #[test]
    fn exit_state_awaiting_inputs_txids_are_sorted() {
        use bark::exit as be;
        use std::collections::HashSet;

        let upstream = be::ExitState::Processing(be::ExitProcessingState {
            tip_height: 5.into(),
            transactions: vec![be::ExitTx {
                txid: txid(0x01),
                status: be::ExitTxStatus::AwaitingInputConfirmation {
                    txids: HashSet::from([txid(0xff), txid(0x02), txid(0x0a)]),
                },
            }],
        });
        let state: ExitState = (&upstream).into();
        let ExitState::Processing { transactions, .. } = state else {
            panic!("expected processing");
        };
        let ExitTxStatus::AwaitingInputConfirmation { txids } = &transactions[0].status else {
            panic!("expected awaiting-input-confirmation");
        };
        let mut sorted = txids.clone();
        sorted.sort_unstable();
        assert_eq!(txids, &sorted);
        assert_eq!(txids.len(), 3);
    }

    #[test]
    fn exit_tx_status_confirmed_keeps_compat_fields() {
        let upstream = bark::exit::ExitTxStatus::Confirmed { block: block_ref(42, 0x0b) };
        let ExitTxStatus::Confirmed { child_txid, block, origin } = (&upstream).into() else {
            panic!("expected confirmed");
        };
        assert_eq!(block, BlockRef::from(&block_ref(42, 0x0b)));
        assert!(child_txid.is_empty());
        assert!(matches!(origin, ExitTxOrigin::Block { confirmed_in } if confirmed_in == block));
    }

    #[test]
    fn exit_state_serde_tags_match_bark_web_domain() {
        // bark-web's domain type expects kebab-case `type` tags with
        // camelCase fields (same shape barkd serves). Guard the contract.
        let state = ExitState::ClaimInProgress {
            tip_height: 12,
            claimable_since: BlockRef { height: 10, hash: "ab".into() },
            claim_txid: "deadbeef".into(),
        };
        assert_eq!(
            serde_json::to_value(&state).unwrap(),
            serde_json::json!({
                "type": "claim-in-progress",
                "tipHeight": 12,
                "claimableSince": { "height": 10, "hash": "ab" },
                "claimTxid": "deadbeef"
            })
        );

        let state = ExitState::VtxoAlreadySpent { tip_height: 3 };
        assert_eq!(
            serde_json::to_value(&state).unwrap(),
            serde_json::json!({ "type": "vtxo-already-spent", "tipHeight": 3 })
        );

        let state = ExitState::VtxoSwept {
            tip_height: 4,
            spent_inputs: vec![OutPoint { txid: "cd".into(), vout: 1 }],
        };
        assert_eq!(
            serde_json::to_value(&state).unwrap(),
            serde_json::json!({
                "type": "vtxo-swept",
                "tipHeight": 4,
                "spentInputs": [{ "txid": "cd", "vout": 1 }]
            })
        );
    }

    // ------------------------------------------------------------------
    // FeeSchedule (item: fee schedule as raw JSON)
    // ------------------------------------------------------------------

    #[test]
    fn debug_info_matches_the_other_encodings() {
        use bitcoin::bip32::{Xpriv, Xpub};

        let secp = bitcoin::secp256k1::Secp256k1::new();
        let xpub = Xpub::from_priv(&secp, &Xpriv::new_master(BtcNetwork::Regtest, &[7; 32]).unwrap());
        let mailbox_id = ark::mailbox::MailboxIdentifier::from_pubkey(xpub.public_key);

        let info: DebugInfo = bark::DebugInfo {
            network: BtcNetwork::Regtest,
            mailbox_id,
            vtxo_xpub: xpub,
        }
        .into();

        assert!(matches!(info.network, Network::Regtest));
        // Same hex as `Wallet::mailbox_identifier`.
        assert_eq!(info.mailbox_id, hex::encode(ark::ProtocolEncoding::serialize(&mailbox_id)));
        assert_eq!(info.vtxo_xpub.parse::<Xpub>().unwrap(), xpub);
    }

    /// The summary folds every pending field into one number, and the total
    /// adds the sats that need a refresh.
    #[test]
    fn balance_summary_conversion() {
        use bitcoin::Amount;

        let sat = Amount::from_sat;
        let balance = BarkBalance {
            spendable: sat(1000),
            needs_refresh: sat(200),
            pending_lightning_send: sat(1),
            claimable_lightning_receive: sat(2),
            pending_in_round: sat(4),
            pending_exit: sat(8),
            pending_board: sat(16),
            pending_arkoor_send: sat(32),
            pending_offboard: sat(64),
        };

        assert_eq!(
            BalanceSummary::from(balance.summary()),
            BalanceSummary {
                spendable_sats: 1000,
                needs_refresh_sats: 200,
                pending_sats: 127,
                total_sats: 1327,
            },
        );
    }

    /// Mirrors bark-json's `EmergencyExitFeeEstimateResponse`: total is the
    /// sum of both legs and the rate is rounded up to whole sat/vB.
    #[test]
    fn emergency_exit_fee_estimate_conversion_and_serde_roundtrip() {
        use bitcoin::{Amount, FeeRate};

        let upstream = bark::exit::ExitFeeEstimate {
            exit_broadcast_fee: Amount::from_sat(4576),
            claim_fee: Amount::from_sat(645),
            // 2.5 sat/vB → reported as 3 sat/vB, like bark-json.
            fee_rate: FeeRate::from_sat_per_kwu(625),
            txs_to_broadcast: 3,
        };

        // The deprecated `fundable` compares the onchain balance against the
        // broadcast leg only.
        let fundable = |balance| {
            EmergencyExitFeeEstimate::new(upstream.clone(), balance).fundable
        };
        assert!(fundable(Some(Amount::from_sat(4576))));
        assert!(!fundable(Some(Amount::from_sat(4575))));
        assert!(!fundable(None));

        let local = EmergencyExitFeeEstimate::new(upstream, None);
        assert_eq!(local.exit_broadcast_fee_sats, 4576);
        assert_eq!(local.claim_fee_sats, 645);
        assert_eq!(local.total_fee_sats, 4576 + 645);
        assert_eq!(local.fee_rate_sat_per_vb, 3);
        assert_eq!(local.txs_to_broadcast, 3);
        assert!(!local.fundable);

        let json = serde_json::to_string(&local).unwrap();
        let back: EmergencyExitFeeEstimate = serde_json::from_str(&json).unwrap();
        assert_eq!(back.total_fee_sats, local.total_fee_sats);
        assert_eq!(back.fee_rate_sat_per_vb, local.fee_rate_sat_per_vb);
        assert_eq!(back.txs_to_broadcast, local.txs_to_broadcast);
        assert_eq!(back.fundable, local.fundable);
    }

    #[test]
    fn fee_schedule_conversion_and_serde_shape() {
        use ark::fees as af;
        use bitcoin::Amount;

        let upstream = af::FeeSchedule {
            // scuttle: free same-server Lightning receives, cheaper sends.
            intra_ark: Some(af::IntraArkFees {
                lightning_send: af::LightningSendFees {
                    min_fee: Amount::ZERO,
                    base_fee: Amount::from_sat(1),
                    ppm_expiry_table: vec![],
                },
                lightning_receive: af::LightningReceiveFees {
                    base_fee: Amount::ZERO,
                    ppm: af::PpmFeeRate(0),
                },
            }),
            board: af::BoardFees {
                min_fee: Amount::from_sat(100),
                base_fee: Amount::from_sat(10),
                ppm: af::PpmFeeRate(4000),
            },
            offboard: af::OffboardFees {
                base_fee: Amount::from_sat(20),
                fixed_additional_vb: 110,
                ppm_expiry_table: vec![af::PpmExpiryFeeEntry {
                    expiry_blocks_threshold: 144,
                    ppm: af::PpmFeeRate(500),
                }],
            },
            refresh: af::RefreshFees {
                base_fee: Amount::from_sat(30),
                ppm_expiry_table: vec![
                    af::PpmExpiryFeeEntry {
                        expiry_blocks_threshold: 144,
                        ppm: af::PpmFeeRate(100),
                    },
                    af::PpmExpiryFeeEntry {
                        expiry_blocks_threshold: 288,
                        ppm: af::PpmFeeRate(200),
                    },
                ],
            },
            lightning_receive: af::LightningReceiveFees {
                base_fee: Amount::from_sat(40),
                ppm: af::PpmFeeRate(600),
            },
            lightning_send: af::LightningSendFees {
                min_fee: Amount::from_sat(50),
                base_fee: Amount::from_sat(5),
                ppm_expiry_table: vec![],
            },
        };

        let ffi: FeeSchedule = (&upstream).into();
        assert_eq!(ffi.board.min_fee_sats, 100);
        assert_eq!(ffi.board.ppm, 4000);
        assert_eq!(ffi.offboard.fixed_additional_vb, 110);
        assert_eq!(ffi.refresh.ppm_expiry_table.len(), 2);
        assert_eq!(ffi.refresh.ppm_expiry_table[1].expiry_blocks_threshold, 288);
        assert_eq!(ffi.refresh.ppm_expiry_table[1].ppm, 200);
        assert_eq!(ffi.lightning_receive.ppm, 600);
        assert_eq!(ffi.lightning_send.min_fee_sats, 50);
        assert!(ffi.lightning_send.ppm_expiry_table.is_empty());

        // Field naming contract for the wasm surface.
        let json = serde_json::to_value(&ffi).unwrap();
        assert_eq!(json["board"]["minFeeSats"], 100);
        assert_eq!(json["refresh"]["ppmExpiryTable"][0]["expiryBlocksThreshold"], 144);
        assert_eq!(json["lightningReceive"]["ppm"], 600);
        assert_eq!(json["lightningSend"]["baseFeeSats"], 5);
        let intra = ffi.intra_ark.as_ref().expect("intra-Ark fees");
        assert_eq!(intra.lightning_send.base_fee_sats, 1);
        assert_eq!(intra.lightning_receive.ppm, 0);
        assert_eq!(json["intraArk"]["lightningSend"]["baseFeeSats"], 1);
        assert_eq!(json["intraArk"]["lightningReceive"]["baseFeeSats"], 0);
    }

    // ------------------------------------------------------------------
    // Onchain data (item: on-chain data, CPFP detection, raw tx hex)
    // ------------------------------------------------------------------

    #[test]
    fn wallet_transaction_conversion_roundtrips_tx_hex() {
        use bitcoin::absolute::LockTime;
        use bitcoin::transaction::Version;
        use std::sync::Arc;

        let tx = bitcoin::Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: vec![],
            output: vec![bitcoin::TxOut {
                value: bitcoin::Amount::from_sat(1234),
                script_pubkey: bitcoin::ScriptBuf::new(),
            }],
        };
        let info = bark::onchain::WalletTxInfo {
            txid: tx.compute_txid(),
            tx: Arc::new(tx.clone()),
            onchain_fees: Some(bitcoin::Amount::from_sat(200)),
            balance_change: bitcoin::SignedAmount::from_sat(-1434),
            confirmation: Some(block_ref(500, 0x11)),
            is_cpfp: true,
        };

        let ffi: WalletTransaction = (&info).into();
        assert_eq!(ffi.txid, tx.compute_txid().to_string());
        assert_eq!(ffi.onchain_fee_sats, Some(200));
        assert_eq!(ffi.balance_change_sats, -1434);
        assert!(ffi.is_cpfp);
        assert_eq!(ffi.confirmation.as_ref().unwrap().height, 500);

        // The hex must decode back to the same transaction.
        let decoded: bitcoin::Transaction =
            bitcoin::consensus::encode::deserialize_hex(&ffi.tx_hex).unwrap();
        assert_eq!(decoded.compute_txid(), tx.compute_txid());
    }

    #[test]
    fn onchain_utxo_local_conversion_and_tags() {
        let upstream = bark::onchain::Utxo::Local(bark::onchain::LocalUtxo {
            outpoint: bitcoin::OutPoint { txid: txid(0x33), vout: 1 },
            amount: bitcoin::Amount::from_sat(5000),
            confirmation_height: None,
        });
        let ffi: OnchainUtxo = (&upstream).into();
        let json = serde_json::to_value(&ffi).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "type": "local",
                "outpoint": { "txid": hex::encode([0x33; 32]), "vout": 1 },
                "amountSats": 5000,
                "confirmationHeight": null
            })
        );
    }

    // ------------------------------------------------------------------
    // RoundFlowKind / RecoveryStatus (bark 0.7.0)
    // ------------------------------------------------------------------

    #[test]
    fn round_flow_kind_maps_every_upstream_variant() {
        use bark::round::RoundFlowKind as B;
        let cases = [
            (B::DelegatedPending, RoundFlowKind::DelegatedPending),
            (B::Pending, RoundFlowKind::Pending),
            (B::Ongoing, RoundFlowKind::Ongoing),
            (B::AwaitingConfirmations, RoundFlowKind::AwaitingConfirmations),
            (B::Failed, RoundFlowKind::Failed),
            (B::Canceled, RoundFlowKind::Canceled),
        ];
        for (upstream, expected) in cases {
            // Tag matches upstream's Display form, so a TS client can compare
            // against `bark` CLI output.
            assert_eq!(
                serde_json::to_value(RoundFlowKind::from(upstream)).unwrap(),
                serde_json::Value::String(upstream.to_string()),
            );
            assert_eq!(RoundFlowKind::from(upstream), expected);
        }
    }

    // ------------------------------------------------------------------
    // BoardFundingInfo (bark 0.9.0)
    // ------------------------------------------------------------------

    fn board_funding_info() -> (BoardFundingInfo, bitcoin::Address) {
        use bitcoin::secp256k1::{PublicKey, Secp256k1, SecretKey};
        let secp = Secp256k1::new();
        let pubkey = PublicKey::from_secret_key(&secp, &SecretKey::from_slice(&[7; 32]).unwrap());
        let address = bitcoin::Address::p2tr(
            &secp, pubkey.x_only_public_key().0, None, bitcoin::Network::Regtest,
        );
        let info = BoardFundingInfo {
            address: address.to_string(),
            expiry_height: 1_234,
            user_pubkey: pubkey.to_string(),
        };
        (info, address)
    }

    /// Upstream only exposes the fields through serde, so a renamed field
    /// there would break `board_psbt` on funding info persisted by callers.
    #[test]
    fn board_funding_info_round_trips_through_bark() {
        let (info, address) = board_funding_info();
        let upstream = bark::BoardFundingAddress::try_from(info.clone()).unwrap();
        assert_eq!(upstream.address(), &address);
        assert_eq!(upstream.expiry_height().to_u32(), 1_234);
        assert_eq!(BoardFundingInfo::try_from(&upstream).unwrap(), info);
    }

    #[test]
    fn board_funding_info_with_a_bad_pubkey_is_refused() {
        let (mut info, _) = board_funding_info();
        info.user_pubkey = "not a pubkey".to_owned();
        let err = bark::BoardFundingAddress::try_from(info)
            .expect_err("a malformed pubkey should be refused");
        assert!(err.message().contains("invalid board funding info"), "{}", err.message());
    }

    #[test]
    fn recovery_status_not_run_has_no_report() {
        let status = RecoveryStatus::from(bark::RecoveryStatus::NotRun);
        assert!(matches!(status, RecoveryStatus::NotRun));
        assert!(status.report().is_none());
        assert_eq!(
            serde_json::to_value(&status).unwrap(),
            serde_json::json!({ "type": "not-run" })
        );
    }

    #[test]
    fn recovery_status_failed_keeps_full_cause_chain() {
        let err = anyhow::anyhow!("mailbox stream closed").context("recovery scan failed");
        let status = RecoveryStatus::from(bark::RecoveryStatus::Failed(err));
        let RecoveryStatus::Failed { message } = &status else {
            panic!("expected Failed, got {status:?}");
        };
        assert!(message.contains("recovery scan failed"), "{message}");
        assert!(message.contains("mailbox stream closed"), "{message}");
        assert!(status.report().is_none());
        assert_eq!(
            serde_json::to_value(&status).unwrap()["type"],
            serde_json::json!("failed")
        );
    }

    #[test]
    fn recovery_status_completed_exposes_report() {
        let status = RecoveryStatus::Completed { report: RecoveryReport::default() };
        assert!(status.report().is_some());
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json["type"], serde_json::json!("completed"));
        assert!(json["report"].is_object(), "{json}");
    }

    #[test]
    fn recovery_status_in_progress_exposes_report() {
        let mut upstream = bark::RecoveryReport::default();
        upstream.push_failed(vtxo_id(1), None);
        let status = RecoveryStatus::from(bark::RecoveryStatus::InProgress(upstream));
        let report = status.report().expect("in-progress carries a report");
        assert_eq!(report.failed.vtxo_ids, vec![vtxo_id(1).to_string()]);
        assert!(!report.is_complete);
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json["type"], serde_json::json!("in-progress"));
        assert!(json["report"].is_object(), "{json}");
    }

    #[test]
    fn recovery_report_maps_not_found_and_abandoned() {
        let mut upstream = bark::RecoveryReport::default();
        upstream.push_not_found(vtxo_id(2));
        for _ in 0..bark::MAX_RECOVERY_ATTEMPTS {
            upstream.push_failed_attempt(vtxo_id(3), Some(bitcoin::Amount::from_sat(1_000)));
        }
        let report = RecoveryReport::from(&upstream);
        assert_eq!(report.not_found.vtxo_ids, vec![vtxo_id(2).to_string()]);
        assert_eq!(report.abandoned.vtxo_ids, vec![vtxo_id(3).to_string()]);
        assert_eq!(report.abandoned.total_sats, 1_000);
        assert!(report.failed.vtxo_ids.is_empty());
        // Abandoned funds may be missing, so the report is not complete.
        assert!(!report.is_complete);
    }

    #[test]
    fn recovery_bucket_ids_are_sorted_as_strings() {
        // `VtxoId` orders vout 2 before vout 10, the strings the other way.
        let id = |vout| ark::VtxoId::from(bitcoin::OutPoint { txid: txid(0x01), vout });
        let mut upstream = bark::RecoveryReport::default();
        upstream.push_not_found(id(2));
        upstream.push_not_found(id(10));
        let report = RecoveryReport::from(&upstream);
        assert_eq!(report.not_found.vtxo_ids, vec![id(10).to_string(), id(2).to_string()]);
    }

    #[test]
    fn log_level_name_matches_serde() {
        for level in log::Level::iter().map(LogLevel::from) {
            assert_eq!(serde_json::to_value(level).unwrap(), level.as_str());
        }
    }

    #[test]
    fn log_level_maps_to_the_same_log_filter() {
        for level in log::Level::iter() {
            assert_eq!(log::LevelFilter::from(LogLevel::from(level)), level.to_level_filter());
        }
    }

    // ------------------------------------------------------------------
    // Payment requests
    // ------------------------------------------------------------------

    /// Destinations from bark's own `PaymentMethod` tests.
    const ARK: &str = "tark1pwh9vsmezqqpjy9akejayl2vvcse6he97rn40g84xrlvrlnhayuuyefrp9nse2y3zqqpjy9akejayl2vvcse6he97rn40g84xrlvrlnhayuuyefrp9nse2yscufs5u";
    const SIGNET_INVOICE: &str = "lntbs100u1p5j0x82sp5d0rwfh7tgrrlwsegy9rx3tzpt36cqwjqza5x4wvcjxjzscfaf6jspp5d8q7354dg3p8h0kywhqq5dq984r8f5en98hf9ln85ug0w8fx6hhsdqqcqzpc9qyysgqyk54v7tpzprxll7e0jyvtxcpgwttzk84wqsfjsqvcdtq47zt2wssxsmtjhz8dka62mdnf9jafhu3l4cpyfnsx449v4wstrwzzql2w5qqs8uh7p";
    const OFFER: &str = "lno1qgsyxjtl6luzd9t3pr62xr7eemp6awnejusgf6gw45q75vcfqqqqqqq2p32x2um5ypmx2cm5dae8x93pqthvwfzadd7jejes8q9lhc4rvjxd022zv5l44g6qah82ru5rdpnpj";
    const LNURL: &str = "LNURL1DP68GURN8GHJ7UM9WFMXJCM99E3K7MF0V9CXJ0M385EKVCENXC6R2C35XVUKXEFCV5MKVV34X5EKZD3EV56NYD3HXQURZEPEXEJXXEPNXSCRVWFNV9NXZCN9XQ6XYEFHVGCXXCMYXYMNSERXFQ5FNS";
    const REGTEST_ADDRESS: &str = "bcrt1qw508d6qejxtdg4y5r3zarvary0c5xw7kygt080";

    fn every_payment_method() -> Vec<bark::movement::PaymentMethod> {
        [
            ("ark", ARK),
            ("bitcoin", "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"),
            ("output-script", "6a0474657374"),
            ("invoice", SIGNET_INVOICE),
            ("offer", OFFER),
            ("lightning-address", "byte@second.tech"),
            ("lnurl", LNURL),
            ("custom", "an arkade address"),
        ]
        .iter()
        .map(|(t, v)| bark::movement::PaymentMethod::from_type_value(t, v).unwrap())
        .collect()
    }

    const PAYEE_SECRET: [u8; 32] = [2; 32];

    /// A signed regtest invoice, created at 1_700_000_000.
    fn regtest_invoice(
        amount_msat: Option<u64>,
        description: Result<&str, bitcoin::hashes::sha256::Hash>,
        expiry_secs: Option<u64>,
    ) -> ark::lightning::Bolt11Invoice {
        use bitcoin::hashes::{sha256, Hash};
        use bitcoin::secp256k1::{Message, Secp256k1, SecretKey};
        use lightning_invoice::{Currency, InvoiceBuilder, PaymentSecret};

        let secp = Secp256k1::new();
        let secret = SecretKey::from_slice(&PAYEE_SECRET).unwrap();
        let builder = InvoiceBuilder::new(Currency::Regtest);
        let builder = match description {
            Ok(d) => builder.description(d.into()),
            Err(hash) => builder.description_hash(hash),
        };
        let mut builder = builder
            .payment_hash(sha256::Hash::hash(b"preimage"))
            .payment_secret(PaymentSecret([42; 32]))
            .duration_since_epoch(std::time::Duration::from_secs(1_700_000_000))
            .min_final_cltv_expiry_delta(144);
        if let Some(secs) = expiry_secs {
            builder = builder.expiry_time(std::time::Duration::from_secs(secs));
        }
        let builder = match amount_msat {
            Some(msat) => builder.amount_milli_satoshis(msat),
            None => builder,
        };
        builder
            .build_signed(|hash: &Message| secp.sign_ecdsa_recoverable(hash, &secret))
            .unwrap()
    }

    #[test]
    fn payment_method_roundtrips_every_kind() {
        for upstream in every_payment_method() {
            let method = PaymentMethod::from(&upstream);
            assert_eq!(method.to_bark().unwrap(), upstream);
            // The same JSON as upstream's, which `Movement.sent_to_addresses`
            // already holds, so TS callers can use one for the other.
            assert_eq!(
                serde_json::to_value(&method).unwrap(),
                serde_json::to_value(&upstream).unwrap(),
            );
        }
    }

    #[test]
    fn a_payment_method_that_does_not_parse_is_refused() {
        assert!(PaymentMethod::Ark { value: "tark1nope".into() }.to_bark().is_err());
        assert!(PaymentMethod::Invoice { value: "byte@second.tech".into() }.to_bark().is_err());
        assert!(PaymentMethod::Bitcoin { value: ARK.into() }.to_bark().is_err());
    }

    #[test]
    fn parsing_errors_keep_their_data() {
        use bark::PaymentMethodParsingError as B;

        let mismatch = PaymentMethodParsingError::from(&B::AmountMismatch {
            expected: bitcoin::Amount::from_sat(1_000),
            got: bitcoin::Amount::from_sat(2_000),
        });
        assert_eq!(
            mismatch,
            PaymentMethodParsingError::AmountMismatch { expected_sats: 1_000, got_sats: 2_000 },
        );
        assert_eq!(
            serde_json::to_value(&mismatch).unwrap(),
            serde_json::json!({ "type": "amount-mismatch", "expectedSats": 1000, "gotSats": 2000 }),
        );

        let server = B::InvalidArkAddress(bark::ArkoorAddressError::ServerMismatch);
        assert_eq!(
            PaymentMethodParsingError::from(&server),
            PaymentMethodParsingError::InvalidArkAddress {
                reason: bark::ArkoorAddressError::ServerMismatch.to_string(),
            },
        );
        assert_eq!(
            PaymentMethodParsingError::from(&B::NetworkMismatch),
            PaymentMethodParsingError::NetworkMismatch,
        );
        assert_eq!(
            PaymentMethodParsingError::from(&B::Unsupported),
            PaymentMethodParsingError::Unsupported,
        );
    }

    #[test]
    fn bolt11_details_are_read_from_the_invoice() {
        use bitcoin::hashes::{sha256, Hash};
        use bitcoin::secp256k1::{PublicKey, Secp256k1, SecretKey};

        let payee = PublicKey::from_secret_key(
            &Secp256k1::new(),
            &SecretKey::from_slice(&PAYEE_SECRET).unwrap(),
        );

        let invoice = regtest_invoice(Some(100_000_001), Ok("coffee"), Some(600));
        let details = Bolt11InvoiceDetails::from(&invoice);
        assert_eq!(details.payment_hash, sha256::Hash::hash(b"preimage").to_string());
        assert_eq!(details.amount_msat, Some(100_000_001));
        assert_eq!(details.amount_sats, Some(100_001), "rounded up");
        assert_eq!(details.description.as_deref(), Some("coffee"));
        assert_eq!(details.description_hash, None);
        assert_eq!(details.payee_pubkey, payee.to_string());
        assert_eq!(details.created_at, 1_700_000_000);
        assert_eq!(details.expires_at, 1_700_000_600);

        // Amountless, with a hashed description and the default expiry.
        let hash = sha256::Hash::hash(b"a long description");
        let details = Bolt11InvoiceDetails::from(&regtest_invoice(None, Err(hash), None));
        assert_eq!(details.amount_msat, None);
        assert_eq!(details.amount_sats, None);
        assert_eq!(details.description, None);
        assert_eq!(details.description_hash, Some(hash.to_string()));
        assert_eq!(details.expires_at, 1_700_000_000 + 3_600);
    }

    #[test]
    fn an_option_says_whether_it_needs_an_amount() {
        let option = |method: bark::movement::PaymentMethod| {
            AvailablePaymentMethod::from(&bark::AvailablePaymentMethod { method, errors: vec![] })
        };

        assert!(option(regtest_invoice(None, Ok(""), None).into()).requires_amount);
        assert!(!option(regtest_invoice(Some(5_000_000), Ok(""), None).into()).requires_amount);
        assert!(option(
            bark::movement::PaymentMethod::from_type_value("bitcoin", REGTEST_ADDRESS).unwrap(),
        ).requires_amount);
    }

    #[test]
    fn a_payment_request_names_its_default_option() {
        let methods = every_payment_method();
        let pick = |kind: &str| {
            methods.iter().find(|m| m.type_str() == kind).unwrap().clone()
        };
        let request = bark::PaymentRequest {
            amount: Some(bitcoin::Amount::from_sat(10_000)),
            label: Some("label".into()),
            message: Some("message".into()),
            options: vec![
                bark::AvailablePaymentMethod { method: pick("bitcoin"), errors: vec![] },
                bark::AvailablePaymentMethod {
                    method: pick("ark"),
                    errors: vec![bark::PaymentMethodParsingError::NetworkMismatch],
                },
                bark::AvailablePaymentMethod { method: pick("invoice"), errors: vec![] },
            ],
        };

        let request = PaymentRequest::from(request);
        assert_eq!(request.amount_sats, Some(10_000));
        assert_eq!(request.label.as_deref(), Some("label"));
        assert_eq!(request.message.as_deref(), Some("message"));
        assert_eq!(request.options.len(), 3);
        assert_eq!(
            request.options[1].errors,
            vec![PaymentMethodParsingError::NetworkMismatch],
        );
        // Ark is preferred, but this one has an error, so lightning wins.
        assert_eq!(request.default_option.as_ref(), Some(&request.options[2]));
    }

    #[test]
    fn a_lightning_payment_result_carries_its_payment_hash() {
        let invoice = regtest_invoice(Some(1_000), Ok(""), None);
        let result = PaymentResult::from(bark::PaymentInitOutput::Lightning(invoice.clone().into()));
        assert_eq!(
            result,
            PaymentResult::Lightning {
                invoice: invoice.to_string(),
                payment_hash: invoice.payment_hash().to_string(),
            },
        );
        assert_eq!(serde_json::to_value(&result).unwrap()["type"], "lightning");
        assert!(serde_json::to_value(&result).unwrap()["paymentHash"].is_string());
        let ark = PaymentResult::from(bark::PaymentInitOutput::Ark(bark::movement::MovementId(7)));
        assert_eq!(ark, PaymentResult::Ark { movement_id: 7 });
        assert_eq!(
            serde_json::to_value(&ark).unwrap(),
            serde_json::json!({ "type": "ark", "movementId": 7 }),
        );
    }

    /// The wasm binding takes the options as JSON, so omitted fields must get
    /// the same defaults as in uniffi.
    #[test]
    fn bip321_uri_options_default_to_ark_and_lightning() {
        let options: Bip321UriOptions = serde_json::from_value(serde_json::json!({})).unwrap();
        assert!(options.ark);
        assert!(options.lightning);
        assert_eq!(options.amount_sats, None);
        assert_eq!(options.onchain_address, None);

        let options: Bip321UriOptions =
            serde_json::from_value(serde_json::json!({ "ark": false })).unwrap();
        assert!(!options.ark);
        assert!(options.lightning);
    }

    #[test]
    fn a_built_uri_lists_its_destinations() {
        use std::str::FromStr;
        use bark::payment_request::BarkBip321Uri;

        let invoice = regtest_invoice(Some(100_000_000), Ok("coffee"), None);
        let text = format!(
            "bitcoin:?amount=0.001&lightning={}&tb={}&ark={}",
            invoice, REGTEST_ADDRESS, ARK,
        );
        let uri = BarkBip321Uri::from_str(&text).unwrap();

        let built = Bip321Uri::from(&uri);
        assert_eq!(built.ark_address.as_deref(), Some(ARK));
        assert_eq!(built.onchain_address.as_deref(), Some(REGTEST_ADDRESS));
        let lightning = built.lightning_invoice.unwrap();
        assert_eq!(lightning.invoice, invoice.to_string());
        assert_eq!(lightning.payment_hash, invoice.payment_hash().to_string());
        assert_eq!(lightning.amount_sats, 100_000);
        assert_eq!(BarkBip321Uri::from_str(&built.uri).unwrap(), uri);
    }
}
