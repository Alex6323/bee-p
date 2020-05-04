//! Type-length-value encoding on top of the protocol messages.

mod header;
mod tlv;

pub(crate) use header::{Header, HEADER_SIZE};
pub(crate) use tlv::Tlv;
