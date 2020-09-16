use bee_transaction::prelude::{Address, Hash, Input, Output, UnsignedTransaction};

#[test]
fn input_to_bincode_serialization() {
    let value = Input::new(Hash([1u8; 32]), 2);
    let bin = bincode::serialize(&value).unwrap();

    let mut expected = vec![1u8; 34];
    expected[0] = 0;
    expected[33] = 2;
    assert_eq!(bin, expected);
}

#[test]
fn output_to_bincode_serialization() {
    let address = Address::from_ed25519_bytes([2; 32]);
    let value = Output::new(address, 18446744073709551615);
    let bin= bincode::serialize(&value).unwrap();
    
    let expected = vec![0, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 255, 255, 255, 255, 255, 255, 255, 255];
    assert_eq!(bin, expected);
}

#[test]
fn unsigned_transaction_to_bincode_serialization() {
    let mut inputs = Vec::new();
    inputs.push(Input::new(Hash([1u8; 32]), 2));
    inputs.push(Input::new(Hash([1u8; 32]), 2));
    let address = Address::from_ed25519_bytes([2; 32]);
    let mut outputs = Vec::new();
    outputs.push(Output::new(address, 18446744073709551615));
    let value = UnsignedTransaction {
        inputs,
        outputs,
        payload: None,
    }; 
    let bin = bincode::serialize(&value).unwrap();

    println!("{:?}", bin);
}
