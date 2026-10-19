//! Namespace-level functions for Bark FFI

use anyhow::Context;
use bark::ark;
use bip39::Mnemonic;

use crate::error::Error;
use crate::types::Bolt11InvoiceDetails;

/// Generate a new 12-word BIP39 mnemonic
#[cfg_attr(feature = "uniffi", uniffi::export)]
pub fn generate_mnemonic() -> Result<String, Error> {
    let mnemonic = Mnemonic::generate(12).context("Failed to generate mnemonic")?;
    Ok(mnemonic.to_string())
}

/// Validate a BIP39 mnemonic phrase
#[cfg_attr(feature = "uniffi", uniffi::export)]
pub fn validate_mnemonic(mnemonic: String) -> Result<bool, Error> {
    match Mnemonic::parse(mnemonic.trim()) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Validate an Ark address (basic format check only)
///
/// This only validates the format of the address, not whether it belongs
/// to a specific Ark server. For full validation against a connected server,
/// use Wallet::validate_arkoor_address() instead.
#[cfg_attr(feature = "uniffi", uniffi::export)]
pub fn validate_ark_address(address: String) -> Result<bool, Error> {
    match address.parse::<ark::Address>() {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Decode a BOLT11 invoice, for example to show its amount, description,
/// payee and expiry before paying it.
///
/// Only decodes: it does not check the invoice's network or whether it has
/// expired. `Wallet::parse_payment_request` checks the network.
#[cfg_attr(feature = "uniffi", uniffi::export)]
pub fn decode_bolt11_invoice(invoice: String) -> Result<Bolt11InvoiceDetails, Error> {
    let invoice: ark::lightning::Bolt11Invoice = invoice
        .trim()
        .parse()
        .context("invalid BOLT11 invoice")?;
    Ok((&invoice).into())
}

/// Extract a signed transaction from a PSBT
///
/// Takes a base64-encoded PSBT and extracts the final signed transaction.
/// This is useful after signing a PSBT (e.g., from drain_exits) before broadcasting.
///
/// # Arguments
///
/// * `psbt_base64` - Base64-encoded PSBT string
///
/// # Returns
///
/// Hex-encoded signed transaction ready for broadcasting
#[cfg_attr(feature = "uniffi", uniffi::export)]
pub fn extract_tx_from_psbt(psbt_base64: String) -> Result<String, Error> {
    use bitcoin::consensus::encode::serialize_hex;
    use bitcoin::psbt::Psbt;
    use std::str::FromStr;

    let psbt = Psbt::from_str(&psbt_base64).context("invalid PSBT")?;

    let tx = psbt.extract_tx().context("Failed to extract transaction")?;

    Ok(serialize_hex(&tx))
}

/// Default for `Config.vtxo_key_gap_limit`: the run of consecutive unused
/// seed-derived VTXO key indices a scan crosses before concluding a VTXO
/// isn't ours.
#[cfg_attr(feature = "uniffi", uniffi::export)]
pub fn default_vtxo_key_gap_limit() -> u32 {
    bark::DEFAULT_VTXO_KEY_GAP_LIMIT
}

/// Largest value `Config.vtxo_key_gap_limit` (or an import/recovery
/// `gap_limit` override) accepts. A scan that matches nothing runs the limit
/// to its end, so an unbounded limit is unbounded work.
#[cfg_attr(feature = "uniffi", uniffi::export)]
pub fn max_vtxo_key_gap_limit() -> u32 {
    bark::MAX_VTXO_KEY_GAP_LIMIT
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 10_000 sat signet invoice with an empty description, from bark's
    /// own tests.
    const SIGNET_INVOICE: &str = "lntbs100u1p5j0x82sp5d0rwfh7tgrrlwsegy9rx3tzpt36cqwjqza5x4wvcjxjzscfaf6jspp5d8q7354dg3p8h0kywhqq5dq984r8f5en98hf9ln85ug0w8fx6hhsdqqcqzpc9qyysgqyk54v7tpzprxll7e0jyvtxcpgwttzk84wqsfjsqvcdtq47zt2wssxsmtjhz8dka62mdnf9jafhu3l4cpyfnsx449v4wstrwzzql2w5qqs8uh7p";

    #[test]
    fn decodes_a_bolt11_invoice() {
        let invoice: ark::lightning::Bolt11Invoice = SIGNET_INVOICE.parse().unwrap();

        let details = decode_bolt11_invoice(format!("  {}\n", SIGNET_INVOICE)).unwrap();
        assert_eq!(details.payment_hash, invoice.payment_hash().to_string());
        assert_eq!(details.amount_msat, Some(10_000_000));
        assert_eq!(details.amount_sats, Some(10_000));
        assert_eq!(details.description.as_deref(), Some(""));
        assert_eq!(details.created_at, invoice.duration_since_epoch().as_secs());
        assert!(details.expires_at > details.created_at);
    }

    #[test]
    fn refuses_anything_but_a_bolt11_invoice() {
        let offer = "lno1qgsyxjtl6luzd9t3pr62xr7eemp6awnejusgf6gw45q75vcfqqqqqqq2p32x2um5ypmx2cm5dae8x93pqthvwfzadd7jejes8q9lhc4rvjxd022zv5l44g6qah82ru5rdpnpj";
        for input in [offer, "byte@second.tech", "lntbs100u1nope", ""] {
            let err = decode_bolt11_invoice(input.to_owned())
                .expect_err(&format!("{:?} decoded", input));
            assert!(err.message().contains("invalid BOLT11 invoice"), "{}", err.message());
        }
    }
}
