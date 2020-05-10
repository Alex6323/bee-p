use crate::transaction::rand_trits_field;

use bee_bundle::Hash;
use bee_protocol::{Milestone, MilestoneIndex};

pub fn clone_ms(ms: &Milestone) -> Milestone {
    Milestone::new(ms.hash().clone(), ms.index())
}

pub fn create_random_milestone(index: MilestoneIndex) -> Milestone {
    Milestone::new(rand_trits_field::<Hash>(), index)
}
