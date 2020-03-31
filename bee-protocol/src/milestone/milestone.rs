use bee_bundle::Hash;

pub type MilestoneIndex = u32;

/// TODO builder ?

#[derive(Debug, Clone)]
pub struct Milestone {
    pub hash: Hash,
    pub index: MilestoneIndex,
}
