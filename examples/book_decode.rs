

use std::fs::File;

use ola_lang_abi::Abi;

fn main() {
    // Parse ABI JSON file
    let abi: Abi = {
        let file = File::open("examples/BookExample.json").expect("failed to open ABI file");

        serde_json::from_reader(file).expect("failed to parse ABI")
    };


    let data = vec![120553111, 7, 60, 5, 111, 108, 97, 118, 109];

    // Decode
    let (func, decoded_data) = abi
        .decode_input_from_slice(&data).unwrap();

    println!("decode function {:?}\n data {:?}", func.name, decoded_data);
}
