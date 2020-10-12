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

mod indexation;
mod milestone;

pub mod transaction;

pub use indexation::Indexation;
pub use milestone::Milestone;
pub use transaction::Transaction;

use bee_common_ext::packable::{Error as PackableError, Packable, Read, Write};

use serde::{Deserialize, Serialize};

use alloc::boxed::Box;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Payload {
    Transaction(Box<Transaction>),
    Milestone(Box<Milestone>),
    Indexation(Box<Indexation>),
}

impl Packable for Payload {
    fn packed_len(&self) -> usize {
        0u32.packed_len()
            + match self {
                Self::Transaction(transaction) => transaction.packed_len(),
                Self::Milestone(milestone) => milestone.packed_len(),
                Self::Indexation(indexation) => indexation.packed_len(),
            }
    }

    fn pack<W: Write>(&self, buf: &mut W) -> Result<(), PackableError> {
        match self {
            Self::Transaction(transaction) => {
                0u32.pack(buf)?;
                transaction.pack(buf)?;
            }
            Self::Milestone(milestone) => {
                1u32.pack(buf)?;
                milestone.pack(buf)?;
            }
            Self::Indexation(indexation) => {
                2u32.pack(buf)?;
                indexation.pack(buf)?;
            }
        }

        Ok(())
    }

    fn unpack<R: Read>(buf: &mut R) -> Result<Self, PackableError>
    where
        Self: Sized,
    {
        Ok(match u32::unpack(buf)? {
            0 => Self::Transaction(Box::new(Transaction::unpack(buf)?)),
            1 => Self::Milestone(Box::new(Milestone::unpack(buf)?)),
            2 => Self::Indexation(Box::new(Indexation::unpack(buf)?)),
            _ => unreachable!(),
        })
    }
}
