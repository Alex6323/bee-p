#[macro_use]
extern crate serde;

mod bundle;
mod constants;
mod incoming_bundle_builder;
mod outgoing_bundle_builder;
mod transaction;

pub use crate::bundle::{
    Bundle,
    Transactions,
};
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
    Nonce,
    Payload,
    Tag,
    Timestamp,
    Transaction,
    TransactionBuilder,
    TransactionBuilderError,
    TransactionBuilders,
    TransactionField,
    Value,
};

pub use constants::{
    ADDRESS_TRIT_LEN,
    HASH_TRIT_LEN,
    IOTA_SUPPLY,
    NONCE_TRIT_LEN,
    PAYLOAD_TRIT_LEN,
    TAG_TRIT_LEN,
    TRANSACTION_BYTE_LEN,
    TRANSACTION_TRIT_LEN,
    TRANSACTION_TRYT_LEN,
};
