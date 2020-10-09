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

use crate::atomic::packable::{Buf, BufMut, Packable};

use serde::{Deserialize, Serialize};

use alloc::boxed::Box;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Payload {
    Transaction(Box<Transaction>),
    Milestone(Box<Milestone>),
    Indexation(Box<Indexation>),
}

impl Packable for Payload {
    fn len_bytes(&self) -> usize {
        0u32.len_bytes()
            + match self {
                Self::Transaction(transaction) => transaction.len_bytes(),
                Self::Milestone(milestone) => milestone.len_bytes(),
                Self::Indexation(indexation) => indexation.len_bytes(),
            }
    }

    fn pack<B: BufMut>(&self, buffer: &mut B) {
        match self {
            Self::Transaction(transaction) => {
                0u32.pack(buffer);
                transaction.pack(buffer);
            }
            Self::Milestone(milestone) => {
                1u32.pack(buffer);
                milestone.pack(buffer);
            }
            Self::Indexation(indexation) => {
                2u32.pack(buffer);
                indexation.pack(buffer);
            }
        }
    }

    fn unpack<B: Buf>(buffer: &mut B) -> Self {
        match u32::unpack(buffer) {
            0 => Self::Transaction(Box::new(Transaction::unpack(buffer))),
            1 => Self::Milestone(Box::new(Milestone::unpack(buffer))),
            2 => Self::Indexation(Box::new(Indexation::unpack(buffer))),
            _ => unreachable!(),
        }
    }
}
