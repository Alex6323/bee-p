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

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

use digest::{generic_array::GenericArray, Digest};
use serde::{Deserialize, Serialize};

use alloc::{boxed::Box, string::String};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Indexation {
    index: String,
    data: Box<[u8]>,
}

impl Indexation {
    pub fn new(index: String, data: Box<[u8]>) -> Self {
        Self { index, data }
    }

    pub fn index(&self) -> &String {
        &self.index
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn hash<D: Digest>(&self, digest: &mut D) -> HashedIndex<D> {
        digest.update(self.index.as_bytes());
        HashedIndex(digest.finalize_reset())
    }
}

impl Packable for Indexation {
    type Error = Error;

    fn packed_len(&self) -> usize {
        0u32.packed_len() + self.index.as_bytes().len() + 0u32.packed_len() + self.data.len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        (self.index.as_bytes().len() as u32).pack(writer)?;
        writer.write_all(self.index.as_bytes())?;

        (self.data.len() as u32).pack(writer)?;
        writer.write_all(&self.data)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let index_len = u32::unpack(reader)? as usize;
        let mut index_bytes = vec![0u8; index_len];
        reader.read_exact(&mut index_bytes)?;

        let data_len = u32::unpack(reader)? as usize;
        let mut data_bytes = vec![0u8; data_len];
        reader.read_exact(&mut data_bytes)?;

        Ok(Self {
            index: String::from_utf8(index_bytes).map_err(|_| Self::Error::InvalidUtf8String)?,
            data: data_bytes.into_boxed_slice(),
        })
    }
}

pub struct HashedIndex<D: Digest>(GenericArray<u8, <D as Digest>::OutputSize>);

impl<D: Digest> AsRef<[u8]> for HashedIndex<D> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
