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

use bee_common_ext::packable::{Error as PackableError, Packable, Read, Write};
use bee_ternary::{T5B1Buf, TritBuf};

use bytemuck::cast_slice;
use serde::{Deserialize, Serialize};

use alloc::vec::Vec;
use core::convert::{TryFrom, TryInto};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct WotsSignature(Vec<u8>);

impl TryFrom<&TritBuf<T5B1Buf>> for WotsSignature {
    type Error = Error;

    fn try_from(trits: &TritBuf<T5B1Buf>) -> Result<Self, Error> {
        // TODO const
        if trits.len() % 6561 != 0 {
            return Err(Error::InvalidSignature);
        }

        let fragments = trits.len() / 6561;

        if fragments < 1 || fragments > 3 {
            return Err(Error::InvalidSignature);
        }

        Ok(Self(cast_slice(trits.as_i8_slice()).to_vec()))
    }
}

// TODO builder ?
impl WotsSignature {
    pub fn new(trits: &TritBuf<T5B1Buf>) -> Result<Self, Error> {
        trits.try_into()
    }
}

impl Packable for WotsSignature {
    fn packed_len(&self) -> usize {
        0u32.packed_len() + self.0.len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), PackableError> {
        (self.0.len() as u32).pack(writer)?;
        writer.write_all(&self.0)?;

        Ok(())
    }

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, PackableError>
    where
        Self: Sized,
    {
        let bytes_len = u32::unpack(reader)? as usize;
        let mut bytes = vec![0u8; bytes_len];
        reader.read_exact(&mut bytes)?;

        Ok(Self(bytes))
    }
}
