extern crate rand;

use bee_bundle::{
    Address,
    Hash,
    Index,
    Nonce,
    Payload,
    Tag,
    Timestamp,
    Transaction,
    TransactionBuilder,
    TransactionField,
    Value,
};
use bee_ternary::{
    T1B1Buf,
    TritBuf,
    Trits,
    T1B1,
};

use rand::Rng;

pub fn rand_trits_field<T: TransactionField>() -> T
// Bit weird, but these constraints permit generating random trits for any transaction field type
where
    T::Inner: ToOwned<Owned = TritBuf>,
    TritBuf: std::borrow::Borrow<T::Inner>,
{
    const TRIT_SET: &[i8] = &[-1, 0, 1];
    let mut rng = rand::thread_rng();

    let raw_buffer: Vec<i8> = (0..T::trit_len())
        .map(|_| {
            let idx = rng.gen_range(0, TRIT_SET.len());
            TRIT_SET[idx]
        })
        .collect();

    let trits = Trits::<T1B1>::try_from_raw(raw_buffer.as_slice(), T::trit_len())
        .unwrap()
        .to_buf::<T1B1Buf>();
    T::from_inner_unchecked(trits)
}

pub fn clone_tx(tx: &Transaction) -> Transaction {
    let builder = TransactionBuilder::new()
        .with_payload(tx.payload().clone())
        .with_address(tx.address().clone())
        .with_value(tx.value().clone())
        .with_obsolete_tag(tx.obsolete_tag().clone())
        .with_timestamp(tx.timestamp().clone())
        .with_index(tx.index().clone())
        .with_last_index(tx.last_index().clone())
        .with_tag(tx.tag().clone())
        .with_attachment_ts(tx.attachment_ts().clone())
        .with_bundle(tx.bundle().clone())
        .with_trunk(tx.trunk().clone())
        .with_branch(tx.branch().clone())
        .with_attachment_lbts(tx.attachment_lbts().clone())
        .with_attachment_ubts(tx.attachment_ubts().clone())
        .with_nonce(tx.nonce().clone());

    builder.build().unwrap()
}

pub fn create_random_tx() -> (Hash, Transaction) {
    let builder = TransactionBuilder::new()
        .with_payload(rand_trits_field::<Payload>())
        .with_address(rand_trits_field::<Address>())
        .with_value(Value::from_inner_unchecked(0))
        .with_obsolete_tag(rand_trits_field::<Tag>())
        .with_timestamp(Timestamp::from_inner_unchecked(0))
        .with_index(Index::from_inner_unchecked(0))
        .with_last_index(Index::from_inner_unchecked(0))
        .with_tag(rand_trits_field::<Tag>())
        .with_attachment_ts(Timestamp::from_inner_unchecked(0))
        .with_bundle(rand_trits_field::<Hash>())
        .with_trunk(rand_trits_field::<Hash>())
        .with_branch(rand_trits_field::<Hash>())
        .with_attachment_lbts(Timestamp::from_inner_unchecked(0))
        .with_attachment_ubts(Timestamp::from_inner_unchecked(0))
        .with_nonce(rand_trits_field::<Nonce>());

    (rand_trits_field::<Hash>(), builder.build().unwrap())
}

pub fn create_random_attached_tx(branch: Hash, trunk: Hash) -> (Hash, Transaction) {
    let builder = TransactionBuilder::new()
        .with_payload(rand_trits_field::<Payload>())
        .with_address(rand_trits_field::<Address>())
        .with_value(Value::from_inner_unchecked(0))
        .with_obsolete_tag(rand_trits_field::<Tag>())
        .with_timestamp(Timestamp::from_inner_unchecked(0))
        .with_index(Index::from_inner_unchecked(0))
        .with_last_index(Index::from_inner_unchecked(0))
        .with_tag(rand_trits_field::<Tag>())
        .with_attachment_ts(Timestamp::from_inner_unchecked(0))
        .with_bundle(rand_trits_field::<Hash>())
        .with_trunk(trunk)
        .with_branch(branch)
        .with_attachment_lbts(Timestamp::from_inner_unchecked(0))
        .with_attachment_ubts(Timestamp::from_inner_unchecked(0))
        .with_nonce(rand_trits_field::<Nonce>());

    (rand_trits_field::<Hash>(), builder.build().unwrap())
}
