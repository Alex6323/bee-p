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

use super::WriteBytes;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Indexation {
    index: String,
    data: Box<[u8]>,
}

impl Indexation {
    pub fn new(index: String, data: Box<[u8]>) -> Self {
        Self { index, data }
    }
}

impl WriteBytes for Indexation {
    fn len_bytes(&self) -> usize {
        0u32.len_bytes() + self.index.as_bytes().len() + 0u32.len_bytes() + self.data.len()
    }

    fn write_bytes(&self, buffer: &mut Vec<u8>) {
        (self.index.as_bytes().len() as u32).write_bytes(buffer);
        self.index.as_bytes().write_bytes(buffer);

        (self.data.len() as u32).write_bytes(buffer);
        self.data.as_ref().write_bytes(buffer);
    }
}
