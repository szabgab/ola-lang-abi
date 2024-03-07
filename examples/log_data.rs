use std::fs::File;

use ola_lang_abi::{Abi, FixedArray4};

fn main() {
    // Parse ABI JSON file
    let abi: Abi = {
        let file = File::open("examples/BookExample.json").expect("failed to open ABI file");

        serde_json::from_reader(file).expect("failed to parse ABI")
    };

    let topics = vec![
        FixedArray4([
            876009939773297099,
            9423535973325601276,
            68930750687700470,
            16776232995860792718,
        ]),
        FixedArray4([0, 0, 0, 10]),
        FixedArray4([
            1298737262017568572,
            12445360621592034485,
            13004999764278192581,
            3441866816748036873,
        ]),
    ];

    let data = vec![5, 104, 101, 108, 108, 111];

    // Decode
    let (evt, decoded_data) = abi
        .decode_log_from_slice(&topics, &data)
        .expect("failed decoding log");

    println!("event: {}\ndata: {:?}", evt.name, decoded_data);
}
