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

use crate::atomic::packable::{Buf, BufMut, Packable};

use serde::{Deserialize, Serialize};

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

impl Packable for Indexation {
    fn packed_len(&self) -> usize {
        0u32.packed_len() + self.index.as_bytes().len() + 0u32.packed_len() + self.data.len()
    }

    fn pack<B: BufMut>(&self, buffer: &mut B) {
        (self.index.as_bytes().len() as u32).pack(buffer);
        Self::pack_bytes(self.index.as_bytes(), buffer);

        (self.data.len() as u32).pack(buffer);
        Self::pack_bytes(self.data.as_ref(), buffer);
    }

    fn unpack<B: Buf>(buffer: &mut B) -> Self {
        let index_len = u32::unpack(buffer) as usize;
        let index_vec = Self::unpack_bytes(buffer, index_len);
        let index = String::from_utf8(index_vec).unwrap();

        let data_len = u32::unpack(buffer) as usize;
        let data = Self::unpack_bytes(buffer, data_len).into_boxed_slice();

        Self { index, data }
    }
}
