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

mod curlp;
mod kerl;
mod r#type;

use super::HASH_LENGTH;

pub use curlp::{CurlP, CurlP27, CurlP81, CurlPRounds};
pub use kerl::Kerl;
pub use r#type::SpongeType;

use bee_ternary::{TritBuf, Trits};

use std::ops::DerefMut;

/// The common interface of cryptographic hash functions that follow the sponge construction, and that absorb and return
/// binary-coded, balanced ternary.
pub trait Sponge {
    /// An error indicating that a failure has occured during a sponge operation.
    type Error;

    /// Absorb `input` into the sponge.
    fn absorb(&mut self, input: &Trits) -> Result<(), Self::Error>;

    /// Reset the inner state of the sponge.
    fn reset(&mut self);

    /// Squeeze the sponge into a buffer
    fn squeeze_into(&mut self, buf: &mut Trits) -> Result<(), Self::Error>;

    /// Convenience function using `Sponge::squeeze_into` to return an owned version of the hash.
    fn squeeze(&mut self) -> Result<TritBuf, Self::Error> {
        let mut output = TritBuf::zeros(HASH_LENGTH);
        self.squeeze_into(&mut output)?;
        Ok(output)
    }

    /// Convenience function to absorb `input`, squeeze the sponge into a buffer, and reset the sponge in one go.
    fn digest_into(&mut self, input: &Trits, buf: &mut Trits) -> Result<(), Self::Error> {
        self.absorb(input)?;
        self.squeeze_into(buf)?;
        self.reset();
        Ok(())
    }

    /// Convenience function to absorb `input`, squeeze the sponge, and reset the sponge in one go. Returns an owned
    /// version of the hash.
    fn digest(&mut self, input: &Trits) -> Result<TritBuf, Self::Error> {
        self.absorb(input)?;
        let output = self.squeeze()?;
        self.reset();
        Ok(output)
    }
}

impl<T: Sponge, U: DerefMut<Target = T>> Sponge for U {
    type Error = T::Error;

    fn absorb(&mut self, input: &Trits) -> Result<(), Self::Error> {
        T::absorb(self, input)
    }

    fn reset(&mut self) {
        T::reset(self)
    }

    fn squeeze_into(&mut self, buf: &mut Trits) -> Result<(), Self::Error> {
        T::squeeze_into(self, buf)
    }
}
