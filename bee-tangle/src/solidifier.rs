use crate::{
    tangle,
    Tangle,
};

use std::collections::HashMap;

use async_std::{
    prelude::*,
    sync::Arc,
};

use dashmap::DashMap;
use flume::Receiver;

use bee_bundle::Hash;

pub struct SolidifierState {
    solidifier_recv: Receiver<Hash>,
}

impl SolidifierState {
    pub fn new(solidifier_recv: Receiver<Hash>) -> Self {
        Self { solidifier_recv }
    }
}

/// Attempt to perform solidification upon a vertex (and its approvers).
pub async fn run(mut state: SolidifierState) {
    while let Some(hash) = state.solidifier_recv.next().await {
        let mut stack = vec![hash];

        while let Some(hash) = stack.pop() {
            if let Some(v) = tangle().vertices.get(&hash).map(|r| r.value().get_ref_to_inner()) {
                if tangle().is_solid_transaction(v.trunk()) && tangle().is_solid_transaction(v.branch()) {
                    // NOTE: unwrap should be safe since we just added it to the Tangle
                    tangle().vertices.get_mut(&hash).unwrap().set_solid();

                    if let Some(approvers) = tangle().approvers.get(&hash) {
                        let approvers = approvers.value();
                        for approver in approvers {
                            stack.push(*approver);
                        }
                    }
                }
            }
        }
    }
}
