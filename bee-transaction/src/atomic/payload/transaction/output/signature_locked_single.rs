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

use crate::atomic::payload::transaction::Address;

use serde::{Deserialize, Serialize};

use core::num::NonZeroU64;

use super::WriteBytes;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SignatureLockedSingleOutput {
    address: Address,
    amount: NonZeroU64,
}

impl SignatureLockedSingleOutput {
    pub fn new(address: Address, amount: NonZeroU64) -> Self {
        Self { address, amount }
    }

    pub fn address(&self) -> &Address {
        &self.address
    }

    pub fn amount(&self) -> NonZeroU64 {
        self.amount
    }
}

impl WriteBytes for SignatureLockedSingleOutput {
    fn len_bytes(&self) -> usize {
        self.address.len_bytes() + u64::from(self.amount).len_bytes()
    }
    fn write_bytes(&self, buffer: &mut Vec<u8>) {
        self.address.write_bytes(buffer);
        u64::from(self.amount).write_bytes(buffer);
    }
}
