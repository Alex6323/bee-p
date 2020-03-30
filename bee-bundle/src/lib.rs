#[macro_use]
extern crate serde;

mod bundle;
mod constants;
mod incoming_bundle_builder;
mod outgoing_bundle_builder;
mod transaction;

pub use crate::bundle::Bundle;
pub use incoming_bundle_builder::{
    IncomingBundleBuilder,
    IncomingBundleBuilderError,
};
pub use outgoing_bundle_builder::{
    OutgoingBundleBuilder,
    OutgoingBundleBuilderError,
};
pub use transaction::{
    Address,
    Hash,
    Index,
    Milestone,
    Nonce,
    Payload,
    Tag,
    Timestamp,
    Transaction,
    TransactionBuilder,
    TransactionBuilderError,
    TransactionBuilders,
    TransactionField,
    Transactions,
    Value,
};

pub use constants::{
    ADDRESS_TRIT_LEN,
    HASH_TRIT_LEN,
    NONCE_TRIT_LEN,
    PAYLOAD_TRIT_LEN,
    TAG_TRIT_LEN,
};
