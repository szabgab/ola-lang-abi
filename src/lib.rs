//! Ethereum Smart Contracts ABI (abstract binary interface) utility library.

mod abi;
mod params;
mod types;
mod values;

pub use abi::*;
pub use params::*;
pub use types::*;
pub use values::*;

use wasm_bindgen::prelude::*;

//use abi::Abi;
use wasm_bindgen::JsValue;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &JsValue);
}

#[wasm_bindgen]
pub fn decode_abi_wrapper(file_content: &[u8], data: &[u64]) -> Result<JsValue, JsValue> {
    let abi: Abi = serde_json::from_slice(file_content)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse decode ABI: {:?}", e)))?;
    let decoded_data = abi
        .decode_input_from_slice(data)
        .map_err(|e| JsValue::from_str(&format!("Error decoding input: {:?}", e)))?;

    log(&JsValue::from_str(&format!(
        "Decoded function name from js: {:#?}",
        decoded_data.0
    )));
    log(&JsValue::from_str(&format!(
        "Decoded function data from js: {:#?}",
        decoded_data.1
    )));

    let func_result_jsvalue = serde_wasm_bindgen::to_value(&decoded_data).map_err(|e| {
        JsValue::from_str(&format!(
            "Error converting decode result to JsValue: {:?}",
            e
        ))
    })?;

    Ok(func_result_jsvalue)
}

#[wasm_bindgen]
pub fn decode_input_from_js(file_content: &[u8], data: &[u64]) -> Result<JsValue, JsValue> {
    log(&JsValue::from_str(&format!(
        "Received data length of encode input from js: {}",
        data.len()
    )));
    for (i, &value) in data.iter().enumerate() {
        log(&JsValue::from_str(&format!(
            "Received data element of encode input from js at index {}: {}",
            i, value
        )));
    }
    decode_abi_wrapper(file_content, data)
}

use serde_wasm_bindgen;
pub fn encode_abi_wrapper(
    file_content: &[u8],
    signature: &str,
    value: &[Value],
) -> Result<JsValue, JsValue> {
    let abi: Abi = serde_json::from_slice(file_content)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse encode ABI: {:?}", e)))?;

    let input = abi
        .encode_input_with_signature(signature, &value)
        .map_err(|e| JsValue::from_str(&format!("Error decoding input: {:?}", e)))?;

    log(&JsValue::from_str(&format!(
        "Encoded function data input from js: {:#?}",
        input
    )));

    let result_jsvalue = serde_wasm_bindgen::to_value(&input).map_err(|e| {
        JsValue::from_str(&format!(
            "Error converting encode result to JsValue: {:?}",
            e
        ))
    })?;

    Ok(result_jsvalue)
}

#[wasm_bindgen]
pub fn encode_input_from_js(
    file_content: &[u8],
    signature: &str,
    params: JsValue,
) -> Result<JsValue, JsValue> {
    log(&JsValue::from_str(&format!(
        "Received signature of encode input from js: {}",
        signature
    )));

    let params: Vec<Value> = serde_wasm_bindgen::from_value(params)?;

    encode_abi_wrapper(file_content, signature, &params)
}
