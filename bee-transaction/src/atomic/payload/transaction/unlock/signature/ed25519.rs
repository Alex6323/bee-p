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

use alloc::vec::Vec;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Ed25519Signature {
    public_key: [u8; 32],
    signature: Vec<u8>,
}

impl Ed25519Signature {
    pub fn new(public_key: [u8; 32], signature: Vec<u8>) -> Self {
        Self { public_key, signature }
    }

    pub fn public_key(&self) -> &[u8; 32] {
        &self.public_key
    }

    pub fn signature(&self) -> &Vec<u8> {
        &self.signature
    }
}

impl Packable for Ed25519Signature {
    fn len_bytes(&self) -> usize {
        32 + 64
    }

    fn pack<B: BufMut>(&self, buffer: &mut B) {
        Self::pack_bytes(self.public_key.as_ref(), buffer);
        Self::pack_bytes(self.signature.as_slice(), buffer);
    }

    fn unpack<B: Buf>(buffer: &mut B) -> Self {
        let public_key_vec = Self::unpack_bytes(buffer, 32);
        let public_key = unsafe { *(public_key_vec.as_slice() as *const [u8] as *const [u8; 32]) };

        let signature = Self::unpack_bytes(buffer, 64);

        Self { public_key, signature }
    }
}
