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

pub mod indexation;
pub mod milestone;
pub mod transaction;

use indexation::Indexation;
use milestone::Milestone;
use transaction::Transaction;

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use serde::{Deserialize, Serialize};

use alloc::boxed::Box;

#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Payload {
    Transaction(Box<Transaction>),
    Milestone(Box<Milestone>),
    Indexation(Box<Indexation>),
}

impl From<Transaction> for Payload {
    fn from(payload: Transaction) -> Self {
        Self::Transaction(Box::new(payload))
    }
}

impl From<Milestone> for Payload {
    fn from(payload: Milestone) -> Self {
        Self::Milestone(Box::new(payload))
    }
}

impl From<Indexation> for Payload {
    fn from(payload: Indexation) -> Self {
        Self::Indexation(Box::new(payload))
    }
}

impl Packable for Payload {
    type Error = Error;

    fn packed_len(&self) -> usize {
        match self {
            Self::Transaction(payload) => 0u32.packed_len() + payload.packed_len(),
            Self::Milestone(payload) => 1u32.packed_len() + payload.packed_len(),
            Self::Indexation(payload) => 2u32.packed_len() + payload.packed_len(),
        }
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        match self {
            Self::Transaction(payload) => {
                0u32.pack(writer)?;
                payload.pack(writer)?;
            }
            Self::Milestone(payload) => {
                1u32.pack(writer)?;
                payload.pack(writer)?;
            }
            Self::Indexation(payload) => {
                2u32.pack(writer)?;
                payload.pack(writer)?;
            }
        }

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(match u32::unpack(reader)? {
            0 => Self::Transaction(Box::new(Transaction::unpack(reader)?)),
            1 => Self::Milestone(Box::new(Milestone::unpack(reader)?)),
            2 => Self::Indexation(Box::new(Indexation::unpack(reader)?)),
            _ => return Err(Self::Error::InvalidVariant),
        })
    }
}
