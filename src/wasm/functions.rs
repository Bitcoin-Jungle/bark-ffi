#![allow(non_snake_case)]

use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::*;

use crate::error::Error;
use crate::functions as core_fns;
use crate::types::Bolt11InvoiceDetails;

fn bark_err(e: Error) -> JsError {
    JsError::new(&e.message())
}

#[wasm_bindgen(js_name = generateMnemonic)]
pub fn generate_mnemonic() -> Result<String, JsError> {
    core_fns::generate_mnemonic().map_err(bark_err)
}

#[wasm_bindgen(js_name = validateMnemonic)]
pub fn validate_mnemonic(mnemonic: String) -> Result<bool, JsError> {
    core_fns::validate_mnemonic(mnemonic).map_err(bark_err)
}

#[wasm_bindgen(js_name = validateArkAddress)]
pub fn validate_ark_address(address: String) -> Result<bool, JsError> {
    core_fns::validate_ark_address(address).map_err(bark_err)
}

/// Decode a BOLT11 invoice, for example to show its amount, description,
/// payee and expiry before paying it. Does not check its network or expiry.
#[wasm_bindgen(js_name = decodeBolt11Invoice)]
pub fn decode_bolt11_invoice(invoice: String) -> Result<Ts<Bolt11InvoiceDetails>, JsError> {
    Ok(core_fns::decode_bolt11_invoice(invoice).map_err(bark_err)?.into_ts()?)
}

#[wasm_bindgen(js_name = extractTxFromPsbt)]
pub fn extract_tx_from_psbt(psbtBase64: String) -> Result<String, JsError> {
    core_fns::extract_tx_from_psbt(psbtBase64).map_err(bark_err)
}

#[wasm_bindgen(js_name = defaultVtxoKeyGapLimit)]
pub fn default_vtxo_key_gap_limit() -> u32 {
    core_fns::default_vtxo_key_gap_limit()
}

#[wasm_bindgen(js_name = maxVtxoKeyGapLimit)]
pub fn max_vtxo_key_gap_limit() -> u32 {
    core_fns::max_vtxo_key_gap_limit()
}
