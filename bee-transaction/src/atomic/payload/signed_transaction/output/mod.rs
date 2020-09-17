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

mod sig_locked_single_deposit;

pub use sig_locked_single_deposit::{Address, SigLockedSingleDeposit};

use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Output {
    SigLockedSingleDeposit(SigLockedSingleDeposit),
}

impl Output {
    pub fn new(address: Address, amount:u64) -> Self {
        Output::SigLockedSingleDeposit(SigLockedSingleDeposit{
            address,
            amount,
        })
    }

    /// Convenient method to get address.
    pub fn address(&self) -> &Address {
        match self {
            Output::SigLockedSingleDeposit(s) => &s.address
        }
    } 

    /// Convenient method to get amount.
    pub fn amount(&self) -> &u64 {
        match self {
            Output::SigLockedSingleDeposit(s) => &s.amount
        }
    } 

}

