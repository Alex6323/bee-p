// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

#![no_std]
#![allow(clippy::module_inception)]

#[macro_use]
extern crate alloc;

mod message;
mod message_id;
mod vertex;

pub mod payload;
pub mod prelude;

pub use message::{Message, MessageBuilder};
pub use message_id::{MessageId, MESSAGE_ID_LENGTH};
pub use vertex::Vertex;

use core::fmt;

#[derive(Debug)]
pub enum Error {
    AmountError,
    CountError,
    NoInput,
    NoOutput,
    DuplicateError,
    // TODO add index
    InvalidIndex,
    InvalidAddress,
    InvalidSignature,
    OrderError,
    HashError,
    PathError,
    MissingField(&'static str),
    SigningError(bee_signing_ext::binary::Error),
    SignatureError(bee_signing_ext::SignatureError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::AmountError => write!(f, "Invalid amount provided."),
            Error::CountError => write!(f, "Invalid count number provided."),
            Error::NoInput => write!(f, "No input provided."),
            Error::NoOutput => write!(f, "No output provided."),
            Error::DuplicateError => write!(f, "The object in the set must be unique."),
            Error::InvalidIndex => write!(f, "Invalid index provided."),
            Error::InvalidAddress => write!(f, "Invalid address provided."),
            Error::InvalidSignature => write!(f, "Invalid signature provided."),
            Error::OrderError => write!(f, "The vector is not sorted by lexicographical order."),
            Error::HashError => write!(f, "The format of provided hash is not correct."),
            Error::PathError => write!(f, "The format of provided BIP32 path is not correct."),
            Error::MissingField(s) => write!(f, "Missing required field: {}.", s),
            Error::SigningError(e) => write!(f, "{}", e),
            Error::SignatureError(e) => write!(f, "{}", e),
        }
    }
}

impl From<bee_signing_ext::binary::Error> for Error {
    fn from(error: bee_signing_ext::binary::Error) -> Self {
        Error::SigningError(error)
    }
}

impl From<bee_signing_ext::SignatureError> for Error {
    fn from(error: bee_signing_ext::SignatureError) -> Self {
        Error::SignatureError(error)
    }
}
