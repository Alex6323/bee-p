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

use crate::atomic::packable::{Error as PackableError, Packable, Read, Write};

use serde::{Deserialize, Serialize};

pub const TRANSACTION_ID_LENGTH: usize = 32;

#[derive(Clone, Copy, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct TransactionId([u8; TRANSACTION_ID_LENGTH]);

impl From<[u8; TRANSACTION_ID_LENGTH]> for TransactionId {
    fn from(bytes: [u8; TRANSACTION_ID_LENGTH]) -> Self {
        Self(bytes)
    }
}

impl TransactionId {
    pub fn new(bytes: [u8; TRANSACTION_ID_LENGTH]) -> Self {
        bytes.into()
    }
}

impl core::fmt::Display for TransactionId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl core::fmt::Debug for TransactionId {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "TransactionId({})", self.to_string())
    }
}

impl Packable for TransactionId {
    fn packed_len(&self) -> usize {
        TRANSACTION_ID_LENGTH
    }

    fn pack<W: Write>(&self, buf: &mut W) -> Result<(), PackableError> {
        buf.write_all(&self.0)?;

        Ok(())
    }

    fn unpack<R: Read>(buf: &mut R) -> Result<Self, PackableError>
    where
        Self: Sized,
    {
        let mut bytes = [0u8; TRANSACTION_ID_LENGTH];
        buf.read_exact(&mut bytes)?;

        Ok(Self(bytes))
    }
}
