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

use crate::tangle;

use bee_message::prelude::MessageId;

use std::collections::HashSet;

use async_std::{
    prelude::*,
    sync::{Arc, Barrier},
    task::block_on,
};
use dashmap::DashMap;
use flume::Receiver;

pub struct SolidifierState {
    solidifier_recv: Receiver<Option<Hash>>,
    drop_barrier: Arc<Barrier>,
}

impl SolidifierState {
    pub fn new(solidifier_recv: Receiver<Option<Hash>>, drop_barrier: Arc<Barrier>) -> Self {
        Self {
            solidifier_recv,
            drop_barrier,
        }
    }

    fn propagate(&self, hash: MessageId) {
        let mut stack = vec![hash];
        let mut already_solid = HashSet::new();

        while let Some(hash) = stack.pop() {
            if !already_solid.contains(&hash) {
                if let Some(v) = tangle().vertices.get(&hash).map(|r| r.value().get_ref_to_inner()) {
                    if tangle().is_solid_transaction(v.parent1()) && tangle().is_solid_transaction(v.parent2()) {
                        // NOTE: unwrap should be safe since we just added it to the Tangle
                        tangle().vertices.get_mut(&hash).unwrap().set_solid();
                        already_solid.insert(hash);

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

    /// Attempt to perform solidification upon a vertex (and its approvers).
    pub async fn run(mut self) {
        while let Ok(hash) = self.solidifier_recv.recv_async().await {
            if let Some(hash) = hash {
                self.propagate(hash);
            } else {
                self.drop_barrier.wait().await;
                break;
            }
        }
    }
}
