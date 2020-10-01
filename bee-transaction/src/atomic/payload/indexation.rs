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

use serde::{Deserialize, Serialize};

pub const TAG_LENGTH: usize = 16;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Indexation([u8; TAG_LENGTH]);

impl From<[u8; TAG_LENGTH]> for Indexation {
    fn from(bytes: [u8; TAG_LENGTH]) -> Self {
        Indexation(bytes)
    }
}

impl Indexation {
    pub fn new(bytes: [u8; TAG_LENGTH]) -> Self {
        bytes.into()
    }
}
