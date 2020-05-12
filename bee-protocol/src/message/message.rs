// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Message trait.

use std::ops::Range;

/// A trait describing the behavior of a message.
///
/// This trait is protocol agnostic and only provides serialization and deserialization to and from byte buffers.
/// It should not be used as is but rather be paired with a higher layer - like a type-length-value encoding - and as
/// such does not provide any safety check on inputs/outputs.
pub(crate) trait Message {
    /// The unique identifier of the message within the protocol.
    const ID: u8;

    /// Returns the size range of the message as it can be compressed.
    fn size_range() -> Range<usize>;

    /// Deserializes a byte buffer into a message.
    ///
    /// # Arguments
    ///
    /// * `bytes`   -   The byte buffer to deserialize from.
    ///
    /// # Panics
    ///
    /// Panics if the provided buffer has an invalid size.
    /// The size of the buffer should be within the range returned by the `size_range` method.
    fn from_bytes(bytes: &[u8]) -> Self;

    /// Returns the size of the message.
    fn size(&self) -> usize;

    /// Serializes a message into a byte buffer.
    ///
    /// # Arguments
    ///
    /// * `bytes`   -   The byte buffer to serialize into.
    ///
    /// # Panics
    ///
    /// Panics if the provided buffer has an invalid size.
    /// The size of the buffer should be equal to the one returned by the `size` method.
    fn into_bytes(self, bytes: &mut [u8]);
}
