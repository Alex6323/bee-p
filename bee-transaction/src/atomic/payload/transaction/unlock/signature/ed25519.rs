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

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Ed25519Signature {
    public_key: [u8; 32],
    // TODO size is 64, change with generic const.
    signature: Box<[u8]>,
}

impl Ed25519Signature {
    pub fn new(public_key: [u8; 32], signature: Box<[u8]>) -> Self {
        Self { public_key, signature }
    }

    pub fn public_key(&self) -> &[u8; 32] {
        &self.public_key
    }

    pub fn signature(&self) -> &[u8] {
        &self.signature
    }
}

impl Packable for Ed25519Signature {
    fn packed_len(&self) -> usize {
        32 + 64
    }

    fn pack<B: BufMut>(&self, buf: &mut B) {
        buf.put_slice(self.public_key.as_ref());
        buf.put_slice(self.signature.as_ref());
    }

    fn unpack<B: Buf>(buf: &mut B) -> Self {
        let mut public_key_bytes = [0u8; 32];
        buf.copy_to_slice(&mut public_key_bytes);

        let mut signature_bytes = vec![0u8; 64];
        buf.copy_to_slice(&mut signature_bytes);

        Self {
            public_key: public_key_bytes,
            signature: signature_bytes.into_boxed_slice(),
        }
    }
}
