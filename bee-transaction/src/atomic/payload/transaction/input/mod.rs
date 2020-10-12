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

mod utxo;

pub use utxo::UTXOInput;

use bee_common_ext::packable::{Error as PackableError, Packable, Read, Write};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum Input {
    UTXO(UTXOInput),
}

impl From<UTXOInput> for Input {
    fn from(input: UTXOInput) -> Self {
        Self::UTXO(input)
    }
}

impl Packable for Input {
    fn packed_len(&self) -> usize {
        match self {
            Self::UTXO(utxo_input) => 0u8.packed_len() + utxo_input.packed_len(),
        }
    }

    fn pack<W: Write>(&self, buf: &mut W) -> Result<(), PackableError> {
        match self {
            Self::UTXO(utxo_input) => {
                0u8.pack(buf)?;
                utxo_input.pack(buf)?;
            }
        }

        Ok(())
    }

    fn unpack<R: Read>(buf: &mut R) -> Result<Self, PackableError>
    where
        Self: Sized,
    {
        Ok(match u8::unpack(buf)? {
            0 => Self::UTXO(UTXOInput::unpack(buf)?),
            _ => unreachable!(),
        })
    }
}
