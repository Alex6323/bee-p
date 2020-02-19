extern crate rand;

use bee_bundle::{
    Address, Hash, Index, Nonce, Payload, Tag, Timestamp, Transaction, TransactionBuilder, Value,
};
use bee_storage::Milestone;

use rand::Rng;

pub fn rand_hash() -> Hash {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ9";
    const HASH_LEN: usize = 81;
    let mut rng = rand::thread_rng();

    let hash_str: String = (0..HASH_LEN)
        .map(|_| {
            let idx = rng.gen_range(0, CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    Hash::from_str(&hash_str)
}

pub fn create_random_tx() -> (Hash, Transaction) {
    let builder = TransactionBuilder::new()
        .with_payload(Payload::zeros())
        .with_address(Address::zeros())
        .with_value(Value(0))
        .with_obsolete_tag(Tag::zeros())
        .with_timestamp(Timestamp(0))
        .with_index(Index(0))
        .with_last_index(Index(0))
        .with_tag(Tag::zeros())
        .with_attachment_ts(Timestamp(0))
        .with_bundle(Hash::zeros())
        .with_trunk(Hash::zeros())
        .with_branch(Hash::zeros())
        .with_attachment_lbts(Timestamp(0))
        .with_attachment_ubts(Timestamp(0))
        .with_nonce(Nonce::from_str("ABCDEF"));

    (rand_hash(), builder.build().unwrap())
}

pub fn create_random_attached_tx(branch: Hash, trunk: Hash) -> (Hash, Transaction) {
    let builder = TransactionBuilder::new()
        .with_payload(Payload::zeros())
        .with_address(Address::zeros())
        .with_value(Value(0))
        .with_obsolete_tag(Tag::zeros())
        .with_timestamp(Timestamp(0))
        .with_index(Index(0))
        .with_last_index(Index(0))
        .with_tag(Tag::zeros())
        .with_attachment_ts(Timestamp(0))
        .with_bundle(Hash::zeros())
        .with_trunk(trunk)
        .with_branch(branch)
        .with_attachment_lbts(Timestamp(0))
        .with_attachment_ubts(Timestamp(0))
        .with_nonce(Nonce::from_str("ABCDEF"));

    (rand_hash(), builder.build().unwrap())
}

pub fn create_random_milestone() -> Milestone {
    Milestone {
        hash: rand_hash(),
        index: 0,
    }
}
