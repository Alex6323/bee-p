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

use serde::{Deserialize, Serialize};

use alloc::boxed::Box;

use super::WriteBytes;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Payload {
    Transaction(Box<Transaction>),
    Milestone(Box<Milestone>),
    Indexation(Box<Indexation>),
}

impl WriteBytes for Payload {
    fn len_bytes(&self) -> usize {
        0u32.len_bytes()
            + match self {
                Self::Transaction(transaction) => transaction.len_bytes(),
                Self::Milestone(milestone) => milestone.len_bytes(),
                Self::Indexation(indexation) => indexation.len_bytes(),
            }
    }

    fn write_bytes(&self, buffer: &mut Vec<u8>) {
        match self {
            Self::Transaction(transaction) => {
                0u32.write_bytes(buffer);
                transaction.write_bytes(buffer);
            }
            Self::Milestone(milestone) => {
                1u32.write_bytes(buffer);
                milestone.write_bytes(buffer);
            }
            Self::Indexation(indexation) => {
                2u32.write_bytes(buffer);
                indexation.write_bytes(buffer);
            }
        }
    }
}
