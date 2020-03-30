//! This is a prototype for [PR #21], the RFC introducing the `Kerl` and `CurlP` hash functions
//! implemented in terms of a common `Sponge` trait.
//!
//! The main focus of this prototype are the [`Sponge`] trait, and the [`CurlP`], and [`Kerl`]
//! types. These are cryptographic hash functions that are sponge constructions implemented in
//! terms of the trait.
//!
//! [PR #21]: https://github.com/iotaledger/bee-rfcs/pull/21

pub mod curlp;
pub mod kerl;

pub use curlp::{
    CurlP,
    CurlP27,
    CurlP81,
};

pub use kerl::Kerl;

use bee_ternary::{
    Trits,
    TritBuf,
};

/// The common interface of cryptographic hash functions that follow the sponge construction,
/// and that absorb and return binary-coded, balanced ternary.
pub trait Sponge {
    /// The expected length of the input to the sponge.
    const IN_LEN: usize;

    /// The length of the hash squeezed from the sponge.
    const OUT_LEN: usize;

    /// An error indicating a that a failure has occured during `absorb`.
    type Error;

    /// Absorb `input` into the sponge.
    fn absorb(&mut self, input: &Trits) -> Result<(), Self::Error>;

    /// Reset the inner state of the sponge.
    fn reset(&mut self);

    /// Squeeze the sponge into a buffer
    fn squeeze_into(&mut self, buf: &mut Trits) -> Result<(), Self::Error>;

    /// Convenience function using `Sponge::squeeze_into` to to return an owned
    /// version of the hash.
    fn squeeze(&mut self) -> Result<TritBuf, Self::Error> {
        let mut output = TritBuf::zeros(Self::OUT_LEN);
        self.squeeze_into(&mut output)?;
        Ok(output)
    }

    /// Convenience function to absorb `input`, squeeze the sponge into a
    /// buffer, and reset the sponge in one go.
    fn digest_into(&mut self, input: &Trits, buf: &mut Trits) -> Result<(), Self::Error> {
        self.absorb(input)?;
        self.squeeze_into(buf)?;
        self.reset();
        Ok(())
    }

    /// Convenience function to absorb `input`, squeeze the sponge, and reset the sponge in one go.
    /// Returns an owned versin of the hash.
    fn digest(&mut self, input: &Trits) -> Result<TritBuf, Self::Error> {
        self.absorb(input)?;
        let output = self.squeeze()?;
        self.reset();
        Ok(output)
    }
}
