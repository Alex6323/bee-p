use bee_bundle::Hash;

pub type MilestoneIndex = u32;

pub struct Milestone {
    pub(crate) hash: Hash,
    pub(crate) index: MilestoneIndex,
}

impl Milestone {
    pub fn new(hash: Hash, index: MilestoneIndex) -> Self {
        Self { hash, index }
    }

    pub fn hash(&self) -> &Hash {
        &self.hash
    }

    pub fn index(&self) -> MilestoneIndex {
        self.index
    }
}
