use bee_transaction::atomic::payload::signed_transaction::input::{Input, UTXOInput};

#[test]
fn input_to_bincode_serialization() {
    let value = Input::UTXO(UTXOInput{
        transaction_id: bee_transaction::atomic::Hash([0u8;32]),
        output_index: 0,
    });
    let x = bincode::serialize(&value).unwrap();
    assert_eq!(x, vec![0u8; 34]);
}