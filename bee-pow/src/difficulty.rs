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

use crate::constants::{DEVNET_DIFFICULTY, HASH_TRIT_LEN, MAINNET_DIFFICULTY, SPAMNET_DIFFICULTY};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Difficulty(pub(self) usize);

impl Difficulty {
    pub fn mainnet() -> Self {
        Self(MAINNET_DIFFICULTY)
    }

    pub fn devnet() -> Self {
        Self(DEVNET_DIFFICULTY)
    }

    pub fn spamnet() -> Self {
        Self(SPAMNET_DIFFICULTY)
    }
}

impl From<usize> for Difficulty {
    fn from(difficulty: usize) -> Self {
        let max_difficulty = HASH_TRIT_LEN;
        if difficulty > max_difficulty {
            Self(max_difficulty)
        } else {
            Self(difficulty)
        }
    }
}

impl std::ops::Deref for Difficulty {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
