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

use crate::atomic::packable::{Buf, BufMut, Packable};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum Input {
    UTXO(UTXOInput),
}

impl Packable for Input {
    fn len_bytes(&self) -> usize {
        match self {
            Self::UTXO(utxo_input) => 0u8.len_bytes() + utxo_input.len_bytes(),
        }
    }

    fn pack_bytes<B: BufMut>(&self, buffer: &mut B) {
        match self {
            Self::UTXO(utxo_input) => {
                0u8.pack_bytes(buffer);
                utxo_input.pack_bytes(buffer);
            }
        }
    }

    fn unpack_bytes<B: Buf>(buffer: &mut B) -> Self {
        match u8::unpack_bytes(buffer) {
            0 => Self::UTXO(UTXOInput::unpack_bytes(buffer)),
            _ => unreachable!(),
        }
    }
}

impl From<UTXOInput> for Input {
    fn from(input: UTXOInput) -> Self {
        Self::UTXO(input)
    }
}
