use std::collections::HashMap;

use async_std::{
    prelude::*,
    sync::{
        Arc,
        Receiver,
    },
};

use bee_bundle::Hash;

pub struct SolidifierState {
    vert_to_approvers: HashMap<Hash, Vec<Hash>>,
    missing_to_approvers: HashMap<Hash, Vec<Arc<Hash>>>,
    unsolid_new: Receiver<Hash>,
}

impl SolidifierState {
    pub fn new(unsolid_new: Receiver<Hash>) -> Self {
        Self {
            vert_to_approvers: HashMap::new(),
            missing_to_approvers: HashMap::new(),
            unsolid_new,
        }
    }
}

pub async fn run(mut state: SolidifierState) {
    while let Some(hash) = state.unsolid_new.next().await {
        // Solidification algorithm here, write back to TANGLE
    }
}

/*
/// Attempt to perform solidification upon a node (and its approvers). This method is private
/// because it is automatically run whenever the tangle is updated with new information
fn try_solidify(&mut self, root: TxHash) {
    // Prevent borrow errors by borrowing the fields independently
    let vertices = &mut self.vertices;
    let txs_to_approvers = &self.txs_to_approvers;

    // The algorithm is recursive, but we don't want to use the stack
    let mut stack = vec![root];
    while let Some(current_vert) = stack.pop() {
        if let Some(approvee_hashes) = vertices
            .get(&current_vert)
            .filter(|v| !v.is_solid())
            .map(|v| v.approvee_hashes())
        {
            if approvee_hashes
                // For each of the current root's approvees...
                .iter()
                // ...ensure that they are all solid...
                .all(|a| {
                    vertices.get(&a).map(|a| a.is_solid()).unwrap_or(false) || a.is_genesis()
                })
            {
                // We can now solidify the current root since we know all approvees are solid
                vertices
                    .get_mut(&current_vert)
                    .unwrap() // Can't fail
                    .set_solid();

                // Now, propagate this information to the approvers of the current root by
                // running the algorithm again for each of them
                for approver in txs_to_approvers
                    .get(&current_vert)
                    .iter()
                    .map(|approvers| approvers.iter())
                    .flatten()
                {
                    // Push the approver to the stack as the next vertex to consider
                    stack.push(*approver);
                }
            }
        }
    }
}
*/
