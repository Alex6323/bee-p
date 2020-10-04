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

use crate::tangle::tangle;
use bee_crypto::ternary::Hash;
use log::info;
use rand::seq::IteratorRandom;
use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    time::Instant,
};

enum Score {
    NonLazy,
    SemiLazy,
    Lazy,
}

// C1: the maximum allowed delta value for the YTRSI of a given transaction in relation to the current LSMI before it
// gets lazy.
const YTRSI_DELTA: u32 = 8;
// C2: the maximum allowed delta value between OTRSI of a given transaction in relation to the current LSMI before it
// gets semi-lazy.
const OTRSI_DELTA: u32 = 13;
// M: the maximum allowed delta value between OTRSI of a given transaction in relation to the current LSMI before it
// gets lazy.
const BELOW_MAX_DEPTH: u32 = 15;
// If the amount of non-lazy tips exceed this limit, remove the parent(s) of the inserted tip to compensate for the
// excess. This rule helps to reduce the amount of tips in the network.
const MAX_LIMIT_NON_LAZY: u8 = 100;
// The maximum time a tip remains in the tip pool after having the first child.
// This rule helps to widen the tangle.
const MAX_AGE_SECONDS_AFTER_FIRST_CHILD: u8 = 3;
// The maximum amount of children a tip is allowed to have before the tip is removed from the tip pool. This rule is
// used to widen the cone of the tangle.
const MAX_NUM_CHILDREN: u8 = 2;

#[derive(Default)]
struct TipMetadata {
    children: HashSet<Hash>,
    time_first_child: Option<Instant>,
}

#[derive(Default)]
pub(crate) struct WurtsTipPool {
    tips: HashMap<Hash, TipMetadata>,
    non_lazy_tips: HashSet<Hash>,
}

impl TipMetadata {
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

impl WurtsTipPool {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn insert(&mut self, tail: Hash, trunk: Hash, branch: Hash) {
        match self.tip_score(&tail) {
            Score::NonLazy => {
                self.non_lazy_tips.insert(tail);
                self.tips.insert(tail, TipMetadata::new());
                self.link_parents_with_child(&tail, &trunk, &branch);
                self.check_retention_rules_for_parents(&trunk, &branch);
            },
            _ => {},
        }
    }

    fn link_parents_with_child(&mut self, hash: &Hash, trunk: &Hash, branch: &Hash) {
        if trunk == branch {
            self.add_child(*trunk, *hash);
        } else {
            self.add_child(*trunk, *hash);
            self.add_child(*branch, *hash);
        }
    }

    fn add_child(&mut self, parent: Hash, child: Hash) {
        match self.tips.entry(parent) {
            Entry::Occupied(mut entry) => {
                let metadata = entry.get_mut();
                metadata.children.insert(child);
                if metadata.time_first_child == None {
                    metadata.time_first_child = Some(Instant::now());
                }
            }
            Entry::Vacant(entry) => {
                let mut metadata = TipMetadata::new();
                metadata.children.insert(child);
                metadata.time_first_child = Some(Instant::now());
                entry.insert(metadata);
            }
        }
    }

    fn check_retention_rules_for_parents(&mut self, trunk: &Hash, branch: &Hash) {
        if trunk == branch {
            self.check_retention_rules_for_parent(trunk);
        } else {
            self.check_retention_rules_for_parent(trunk);
            self.check_retention_rules_for_parent(branch);
        }
    }

    fn check_retention_rules_for_parent(&mut self, parent: &Hash) {
        let should_remove = {
            if self.non_lazy_tips.len() > MAX_LIMIT_NON_LAZY as usize {
                true
            } else if self.tips.get(parent).unwrap().children.len() as u8 > MAX_NUM_CHILDREN {
                true
            } else if self
                .tips
                .get(parent)
                .unwrap()
                .time_first_child
                .unwrap()
                .elapsed()
                .as_secs() as u8
                > MAX_AGE_SECONDS_AFTER_FIRST_CHILD
            {
                true
            } else {
                false
            }
        };
        if should_remove {
            self.tips.remove(parent);
            self.non_lazy_tips.remove(parent);
        }
    }

    pub(crate) fn update_scores(&mut self) {
        let mut to_remove = Vec::new();

        for (tip, _) in &self.tips {
            match self.tip_score(&tip) {
                Score::SemiLazy => {
                    to_remove.push(*tip);
                }
                Score::Lazy => {
                    to_remove.push(*tip);
                }
                _ => {}
            }
        }

        for tip in to_remove {
            self.tips.remove(&tip);
            self.non_lazy_tips.remove(&tip);
        }

        info!("non-lazy {}", self.non_lazy_tips.len());
    }

    fn tip_score(&self, hash: &Hash) -> Score {

        // in case the tip was pruned by the node, consider tip as lazy
        if !tangle().contains(hash) {
            return Score::Lazy;
        }

        let lsmi = *tangle().get_latest_solid_milestone_index();
        let otrsi = *tangle().otrsi(&hash).unwrap();
        let ytrsi = *tangle().ytrsi(&hash).unwrap();

        if (lsmi - ytrsi) > YTRSI_DELTA {
            return Score::Lazy;
        }

        if (lsmi - otrsi) > BELOW_MAX_DEPTH {
            return Score::Lazy;
        }

        if (lsmi - otrsi) > OTRSI_DELTA {
            return Score::SemiLazy;
        }

        Score::NonLazy
    }

    pub fn two_non_lazy_tips(&self) -> Option<(Hash, Hash)> {
        let non_lazy_tips = &self.non_lazy_tips;
        return if non_lazy_tips.is_empty() {
            None
        } else if non_lazy_tips.len() == 1 {
            let tip = non_lazy_tips.iter().next().unwrap();
            Some((*tip, *tip))
        } else if non_lazy_tips.len() == 2 {
            let mut iter = non_lazy_tips.iter();
            Some((*iter.next().unwrap(), *iter.next().unwrap()))
        } else {
            let hashes = non_lazy_tips.iter().choose_multiple(&mut rand::thread_rng(), 2);
            let mut iter = hashes.iter();
            Some((**iter.next().unwrap(), **iter.next().unwrap()))
        };
    }

    pub(crate) fn reduce_tips(&mut self) {
        let mut to_remove = Vec::new();
        for (tip, metadata) in &self.tips {
            let should_remove = {
                if metadata.children.len() == 0 {
                    false
                } else if (metadata.time_first_child.unwrap().elapsed().as_secs() as u8)
                    < MAX_AGE_SECONDS_AFTER_FIRST_CHILD
                {
                    false
                } else {
                    true
                }
            };
            if should_remove {
                to_remove.push(*tip);
            }
        }
        for tip in to_remove {
            self.tips.remove(&tip);
            self.non_lazy_tips.remove(&tip);
        }
    }
}