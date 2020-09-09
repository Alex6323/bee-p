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

mod hash;
mod message;
pub mod payload;

pub use hash::Hash;
pub use message::Message;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid amount provided.")]
    AmountError,
    #[error("Invalid count number provided.")]
    CountError,
    #[error("The length of the object is empty.")]
    EmptyError,
    #[error("The object in the set must be unique.")]
    DuplicateError,
    #[error("The position of index is not correct.")]
    IndexError,
    #[error("The vector is not sorted by lexicographical order.")]
    OrderError,
    #[error("The format of provided hash is not correct.")]
    HashError,
    #[error("The format of provided BIP32 path is not correct.")]
    PathError,
    #[error(transparent)]
    BincodeError(#[from] bincode::Error),
    #[error(transparent)]
    SigningError(#[from] bee_signing_ext::binary::ed25519::Error),
    #[error(transparent)]
    SignatureError(#[from] bee_signing_ext::SignatureError),
}
