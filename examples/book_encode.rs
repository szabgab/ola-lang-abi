use std::fs::File;

use ola_lang_abi::{Abi, Value};

fn main() {
    // Parse ABI from file
    let abi: Abi = {
        let file = File::open("examples/sqrt_prophet_abi.json").expect("failed to open ABI file");

        serde_json::from_reader(file).expect("failed to parse ABI")
    };

    let function_sig = "sqrt_benchmark(u32,u32)";

    let params = vec![Value::U32(60), Value::U32(1)];

    let input = abi
        .encode_input_with_signature(function_sig, &params)
        .unwrap();

    println!("create_book input: {:?}", input);
}
