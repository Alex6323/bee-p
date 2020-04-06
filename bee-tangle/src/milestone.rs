use std::ops::Deref;

/// A wrapper a round a `u32` that represents a milestone index.
pub struct MilestoneIndex(u32);

impl Deref for MilestoneIndex {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u32> for MilestoneIndex {
    fn from(v: u32) -> Self {
        Self(v)
    }
}
