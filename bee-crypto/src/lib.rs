//! This is a prototype for [PR #21], the RFC introducing the `Kerl` and `CurlP` hash functions
//! implemented in terms of a common `Sponge` trait.
//!
//! The main focus of this prototype are the [`Sponge`] trait, and the [`CurlP`], and [`Kerl`]
//! types. These are cryptographic hash functions that are sponge constructions implemented in
//! terms of the trait.
//!
//! [PR #21]: https://github.com/iotaledger/bee-rfcs/pull/21

mod curlp;
mod kerl;
mod sponge;
mod sponge_type;

pub use curlp::{
    CurlP,
    CurlP27,
    CurlP81,
};
pub use kerl::Kerl;
pub use sponge::Sponge;
pub use sponge_type::SpongeType;
