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

pub use bytes::{Buf, BufMut};

pub trait Packable {
    fn packed_len(&self) -> usize;

    fn pack<B: BufMut>(&self, buffer: &mut B);

    fn unpack<B: Buf>(buffer: &mut B) -> Self;
}

macro_rules! impl_packable_for_num {
    ($ty:ident) => {
        impl Packable for $ty {
            fn packed_len(&self) -> usize {
                std::mem::size_of::<$ty>()
            }

            fn pack<B: BufMut>(&self, buffer: &mut B) {
                buffer.put(self.to_le_bytes().as_ref());
            }

            fn unpack<B: Buf>(buffer: &mut B) -> Self {
                let mut bytes = [0; std::mem::size_of::<$ty>()];
                buffer.copy_to_slice(&mut bytes);
                $ty::from_le_bytes(bytes)
            }
        }
    };
}

impl_packable_for_num!(u8);
impl_packable_for_num!(u16);
impl_packable_for_num!(u32);
impl_packable_for_num!(u64);
