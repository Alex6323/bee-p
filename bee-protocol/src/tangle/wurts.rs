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
use log::{error, info};
use rand::seq::IteratorRandom;
use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    time::SystemTime,
};
use std::time::Instant;

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
// the maximum amount of current tips for which "MAX_AGE_SECONDS" and "MAX_NUM_CHILDREN" are checked. if the amount of
// tips exceeds this limit, referenced tips (tips with children) get removed directly to reduce the amount of tips in
// the network.
const RETENTION_LIMIT: u8 = 100;
// the maximum time a tip remains in the tip pool after it was referenced by the first transaction. this is used to
// widen the cone of the tangle. (non-lazy pool)
const MAX_AGE_SECONDS: u8 = 3;
// the maximum amount of children a tip is allowed to have before the tip is removed from the tip pool. this is used to
// widen the cone of the tangle. (non-lazy pool)
const MAX_NUM_CHILDREN: u8 = 2;

#[derive(Default)]
struct TipMetadata {
    children: HashSet<Hash>,
    age_seconds_after_first_child: Option<Instant>,
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

        let is_non_lazy = {
            // if parents are considered as non-lazy, child will be considered as non-lazy too
            if self.non_lazy_tips.contains(&trunk) && self.non_lazy_tips.contains(&branch) {
                true
            } else {
                // in case parents are not present, calculate score of the tip
                match self.tip_score(&tail) {
                    Score::NonLazy => true,
                    _ => false
                }
            }
        };

        if is_non_lazy {
            self.tips.insert(tail, TipMetadata::new());
            self.link_parents_with_child(&tail, &trunk, &branch);
            self.non_lazy_tips.insert(tail);
            self.check_retention_rules_for_parent(&trunk);
            self.check_retention_rules_for_parent(&branch);
        }

    }


    fn link_parents_with_child(&mut self, hash: &Hash, trunk: &Hash, branch: &Hash) {
        self.add_child(*trunk, *hash);
        self.add_child(*branch, *hash);
    }

    fn add_child(&mut self, parent: Hash, child: Hash) {
        match self.tips.entry(parent) {
            Entry::Occupied(mut entry) => {
                let metadata = entry.get_mut();
                metadata.children.insert(child);
            }
            Entry::Vacant(entry) => {
                let mut metadata = TipMetadata::new();
                metadata.children.insert(child);
                metadata.age_seconds_after_first_child = Some(Instant::now());
                entry.insert(metadata);
            }
        }
    }

    fn check_retention_rules_for_parent(&mut self, parent: &Hash) {
        let should_remove = {
            if self.non_lazy_tips.len() > RETENTION_LIMIT as usize {
                true
            } else if self.tips.get(parent).unwrap().children.len() as u8 > MAX_NUM_CHILDREN {
                true
            } else if self.tips.get(parent).unwrap().age_seconds_after_first_child.unwrap().elapsed().as_secs() as u8 > MAX_AGE_SECONDS {
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

    // further optimization: avoid allocations
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

        info!("Available tips (non-lazy): {}", self.non_lazy_tips.len());
    }

    fn tip_score(&self, hash: &Hash) -> Score {
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

    pub fn get_non_lazy_tips(&self) -> Option<(Hash, Hash)> {
        self.select_tips(&self.non_lazy_tips)
    }

    fn select_tips(&self, hashes: &HashSet<Hash>) -> Option<(Hash, Hash)> {
        let mut ret = HashSet::new();

        for i in 1..10 {
            match self.select_tip(hashes) {
                Some(tip) => {
                    ret.insert(tip);
                }
                None => (),
            }
        }

        return if ret.is_empty() {
            None
        } else if ret.len() == 1 {
            let tip = ret.iter().next().unwrap();
            Some((*tip, *tip))
        } else {
            let mut iter = ret.iter();
            Some((*iter.next().unwrap(), *iter.next().unwrap()))
        };
    }

    fn select_tip(&self, hashes: &HashSet<Hash>) -> Option<Hash> {
        if hashes.is_empty() {
            return None;
        }
        Some(*hashes.iter().choose(&mut rand::thread_rng()).unwrap())
    }
}