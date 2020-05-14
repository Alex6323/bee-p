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

pub const TRANSACTION_TRIT_LEN: usize = 8019;
pub const TRANSACTION_TRYT_LEN: usize = TRANSACTION_TRIT_LEN / 3; // 2673
pub const TRANSACTION_BYTE_LEN: usize = TRANSACTION_TRIT_LEN / 5 + 1; // 1604

pub const PAYLOAD_TRIT_LEN: usize = 6561;
pub const ADDRESS_TRIT_LEN: usize = 243;
pub const VALUE_TRIT_LEN: usize = 81;
pub const TAG_TRIT_LEN: usize = 81;
pub const TIMESTAMP_TRIT_LEN: usize = 27;
pub const INDEX_TRIT_LEN: usize = 27;
pub const HASH_TRIT_LEN: usize = 243;
pub const NONCE_TRIT_LEN: usize = 81;

pub const MAINNET_DIFFICULTY: usize = 14;
pub const DEVNET_DIFFICULTY: usize = 9;
pub const SPAMNET_DIFFICULTY: usize = 6;
