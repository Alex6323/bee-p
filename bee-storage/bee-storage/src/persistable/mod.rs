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

use crate::storage::Backend;

pub trait Persistable<S: Backend>: Sized {
    /// This encode method will extend the provided buffer and return ();
    fn write_to(&self, buffer: &mut Vec<u8>);
    /// Decode `slice` and return Self.
    fn read_from(slice: &[u8]) -> Self;
}
