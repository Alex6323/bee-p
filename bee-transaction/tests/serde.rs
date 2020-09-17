use bee_transaction::prelude::{Address, Hash, Input, Output, UnsignedTransaction};

#[test]
fn input_to_json_serde() {
    let value = Input::new(Hash([1u8; 32]), 2);
    let json = serde_json::to_string(&value).unwrap();

    // TODO deserialize and verify
    println!("{}", json);
}

#[test]
fn output_to_json_serde() {
    let address = Address::from_ed25519_bytes([2; 32]);
    let value = Output::new(address, 18446744073709551615);
    let _json = serde_json::to_string(&value).unwrap();
    
    // TODO deserialize and verify
}

#[test]
fn unsigned_transaction_to_json_serde() {
    let mut inputs = Vec::new();
    inputs.push(Input::new(Hash([1u8; 32]), 2));
    inputs.push(Input::new(Hash([3u8; 32]), 4));
    let address = Address::from_ed25519_bytes([2; 32]);
    let mut outputs = Vec::new();
    outputs.push(Output::new(address, 18446744073709551615));
    let value = UnsignedTransaction {
        inputs,
        outputs,
        payload: None,
    }; 
    let _json = serde_json::to_string(&value).unwrap();

    // TODO deserialize and verify
}

