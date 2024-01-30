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
    decode_abi_wrapper(file_content, data)
}

#[wasm_bindgen]
pub fn decode_output_wrapper(file_content: &[u8], signature: &str, data: &[u64]) -> Result<JsValue, JsValue> {
    let abi: Abi = serde_json::from_slice(file_content)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse decode output ABI: {:?}", e)))?;
    let decoded_data = abi
        .decode_output_from_slice(signature, data)
        .map_err(|e| JsValue::from_str(&format!("Error decoding input: {:?}", e)))?;

    let func_result_jsvalue = serde_wasm_bindgen::to_value(&decoded_data).map_err(|e| {
        JsValue::from_str(&format!(
            "Error converting decode output result to JsValue: {:?}",
            e
        ))
    })?;

    Ok(func_result_jsvalue)
}

#[wasm_bindgen]
pub fn decode_output_from_js(file_content: &[u8], signature: &str, data: &[u64]) -> Result<JsValue, JsValue> {
    decode_output_wrapper(file_content, signature, data)
}

use serde_wasm_bindgen;
pub fn encode_abi_wrapper(
    file_content: &[u8],
    signature: &str,
    value: &[Value],
) -> Result<Vec<JsValue>, JsValue> {
    let abi: Abi = serde_json::from_slice(file_content)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse encode ABI: {:?}", e)))?;

    let result = abi
        .encode_input_with_signature(signature, &value)
        .map_err(|e| JsValue::from_str(&format!("Error decoding input: {:?}", e)))?;

    let mut result_js = Vec::with_capacity(result.len());

    for value in result.iter() {
        let bigint_value: u64 = *value;
        let bigint_string = format!("{}", bigint_value);
        result_js.push(JsValue::from_str(&bigint_string));
    }

    Ok(result_js)
}

#[wasm_bindgen]
pub fn encode_input_from_js(
    file_content: &[u8],
    signature: &str,
    params: JsValue,
) -> Result<Vec<JsValue>, JsValue> {

    let params: Vec<Value> = serde_wasm_bindgen::from_value(params)?;

    encode_abi_wrapper(file_content, signature, &params)
}
