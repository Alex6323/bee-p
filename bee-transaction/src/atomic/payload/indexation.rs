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

use crate::atomic::packable::{Error as PackableError, Packable, Read, Write};

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

    fn pack<W: Write>(&self, buf: &mut W) -> Result<(), PackableError> {
        (self.index.as_bytes().len() as u32).pack(buf)?;
        buf.write(self.index.as_bytes())?;

        (self.data.len() as u32).pack(buf)?;
        buf.write(self.data.as_ref())?;

        Ok(())
    }

    fn unpack<R: Read>(buf: &mut R) -> Result<Self, PackableError>
    where
        Self: Sized,
    {
        let index_len = u32::unpack(buf)? as usize;
        let mut index_bytes = vec![0u8; index_len];
        buf.read(&mut index_bytes)?;
        // TODO unwrap ?
        let index = String::from_utf8(index_bytes).unwrap();

        let data_len = u32::unpack(buf)? as usize;
        let mut data = Vec::with_capacity(data_len);
        buf.read(&mut data)?;

        Ok(Self {
            index,
            data: data.into_boxed_slice(),
        })
    }
}
