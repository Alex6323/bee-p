pub(crate) const DEFAULT_RECONNECT_INTERVAL: Seconds = Seconds(60);

use serde::Deserialize;

use std::ops::Deref;

/// A wrapper around a `u64` to represent seconds.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
pub struct Seconds(u64);

impl Seconds {
    /// Returns the corresponding amount of milliseconds.
    pub fn milliseconds(&self) -> u64 {
        self.0 * 1000
    }
}

impl From<u64> for Seconds {
    fn from(seconds: u64) -> Self {
        Self(seconds)
    }
}

impl Deref for Seconds {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
