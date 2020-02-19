extern crate rand;

use bee_bundle::{Hash, Transaction};
use bee_storage::{Milestone, StorageBackend};

use rand::Rng;

pub fn rand_hash() -> bee_bundle::Hash {
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

pub fn create_random_tx() -> (bee_bundle::Hash, bee_bundle::Transaction) {
    let builder = bee_bundle::TransactionBuilder::new()
        .with_payload(bee_bundle::Payload::zeros())
        .with_address(bee_bundle::Address::zeros())
        .with_value(bee_bundle::Value(0))
        .with_obsolete_tag(bee_bundle::Tag::zeros())
        .with_timestamp(bee_bundle::Timestamp(0))
        .with_index(bee_bundle::Index(0))
        .with_last_index(bee_bundle::Index(0))
        .with_tag(bee_bundle::Tag::zeros())
        .with_attachment_ts(bee_bundle::Timestamp(0))
        .with_bundle(Hash::zeros())
        .with_trunk(Hash::zeros())
        .with_branch(Hash::zeros())
        .with_attachment_lbts(bee_bundle::Timestamp(0))
        .with_attachment_ubts(bee_bundle::Timestamp(0))
        .with_nonce(bee_bundle::Nonce::from_str("ABCDEF"));

    (rand_hash(), builder.build().unwrap())
}

pub fn create_random_attached_tx(
    branch: bee_bundle::Hash,
    trunk: bee_bundle::Hash,
) -> (bee_bundle::Hash, bee_bundle::Transaction) {
    let builder = bee_bundle::TransactionBuilder::new()
        .with_payload(bee_bundle::Payload::zeros())
        .with_address(bee_bundle::Address::zeros())
        .with_value(bee_bundle::Value(0))
        .with_obsolete_tag(bee_bundle::Tag::zeros())
        .with_timestamp(bee_bundle::Timestamp(0))
        .with_index(bee_bundle::Index(0))
        .with_last_index(bee_bundle::Index(0))
        .with_tag(bee_bundle::Tag::zeros())
        .with_attachment_ts(bee_bundle::Timestamp(0))
        .with_bundle(Hash::zeros())
        .with_trunk(trunk)
        .with_branch(branch)
        .with_attachment_lbts(bee_bundle::Timestamp(0))
        .with_attachment_ubts(bee_bundle::Timestamp(0))
        .with_nonce(bee_bundle::Nonce::from_str("ABCDEF"));

    (rand_hash(), builder.build().unwrap())
}

pub fn create_random_milestone() -> Milestone {
    Milestone {
        hash: rand_hash(),
        index: 0,
    }
}
