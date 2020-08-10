use bee_transaction::atomic::payload::signed_transaction::{
    Input, UTXOInput,
    SigLockedSingleDeposit, Output, Address,
};

#[test]
fn input_to_bincode_serialization() {
    let value = Input::UTXO(UTXOInput{
        transaction_id: bee_transaction::atomic::Hash([0u8;32]),
        output_index: 0,
    });
    let x = bincode::serialize(&value).unwrap();
    assert_eq!(x, vec![0u8; 34]);
}

#[test]
fn output_to_bincode_serialization() {
    let value = Output::SigLockedSingleDeposit(SigLockedSingleDeposit{
        address: Address::from_ed25519_bytes([0;32]),
        amount: 0,
    });
    let x = bincode::serialize(&value).unwrap();
    let mut expected = vec![0u8; 42];
    expected[1] = 1;
    assert_eq!(x, expected);
}