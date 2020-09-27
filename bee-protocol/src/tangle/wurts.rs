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
use bee_tangle::Tangle;
use bee_ternary::tryte::Tryte::O;
use bee_transaction::Vertex;
use log::{error, info};
use rand::{seq::IteratorRandom};
use std::{
    collections::HashSet,
    time::SystemTime,
};
use std::collections::HashMap;
use std::collections::hash_map::Entry;

enum Score {
    NonLazy,
    SemiLazy,
    Lazy,
}

// C1: the maximum allowed delta value for the YTRSI of a given transaction in relation to the current LSMI before it gets lazy.
const YTRSI_DELTA: u32 = 8;
// C2: the maximum allowed delta value between OTRSI of a given transaction in relation to the current LSMI before it gets semi-lazy.
const OTRSI_DELTA: u32 = 13;
// M: the maximum allowed delta value between OTRSI of a given transaction in relation to the current LSMI before it gets lazy.
const BELOW_MAX_DEPTH: u32 = 15;
// the maximum amount of current tips for which "MAX_AGE_SECONDS" and "MAX_NUM_CHILDREN" are checked. if the amount of tips exceeds this limit,
// referenced tips get removed directly to reduce the amount of tips in the network.
const RETENTION_LIMIT: u8 = 100;
// the maximum time a tip remains in the tip pool after it was referenced by the first transaction. this is used to widen the cone of the tangle. (non-lazy pool)
const MAX_AGE_SECONDS: u8 = 3;
// the maximum amount of children a tip is allowed to have before the tip is removed from the tip pool. this is used to widen the cone of the tangle. (non-lazy pool)
const MAX_NUM_CHILDREN: u8 = 2;

#[derive(Default)]
pub(crate) struct WurtsTipPool {
    tips: HashMap<Hash, Option<SystemTime>>,
    children: HashMap<Hash, HashSet<Hash>>,
    non_lazy_tips: HashSet<Hash>,
}

impl WurtsTipPool {
    pub(crate) fn new() -> Self {
        Self::default()
    }
    // WURTS only supports solid transactions. This function therefore expects that each passed hash is solid.
    // In context of WURTS, a transaction can be considered as a "tip" if:
    // - transaction is solid and has no children
    // - transaction is solid and has only non-solid children
    // - transaction is solid and has solid children but does not exceed the retention rules
    pub(crate) fn insert(&mut self, tail: Hash, trunk: Hash, branch: Hash) {
        // remove referenced tips if retention limit is exceeded
        self.check_retention_limit();
        // remove tips that are too old
        self.check_age_seconds();
        // store tip
        self.store(&tail);
        // link parents with child
        self.add_to_parents(&tail, &trunk, &branch);
        // remove parents that have more than 'MAX_CHILDREN_COUNT' children
        self.check_num_children_of_parents(&trunk, &branch);
    }

    fn check_retention_limit(&mut self) {
        for (tip, _) in self.tips.clone() {
            if self.tips.len() > RETENTION_LIMIT as usize {
                if self.num_children(&tip) > 0 {
                    self.remove_tip(&tip);
                }
            } else {
                break
            }
        }
    }

    fn check_age_seconds(&mut self) {
        for (tip, time) in self.tips.clone() {
            if let Some(time) = time {
                match time.elapsed() {
                    Ok(elapsed) => {
                        if elapsed.as_secs() as u8 > MAX_AGE_SECONDS {
                            self.remove_tip(&tip);
                        }
                    }
                    Err(e) => error!("{:?}", e),
                }
            }
        }
    }

    fn store(&mut self, tip: &Hash) {
        self.tips.insert(*tip, None);
        self.children.insert(*tip, HashSet::new());
    }

    fn add_to_parents(&mut self, hash: &Hash, trunk: &Hash, branch: &Hash) {
        self.add_child(*trunk, *hash);
        self.add_child(*branch, *hash);
    }

    fn add_child(&mut self, parent: Hash, child: Hash) {
        match self.children.entry(parent) {
            Entry::Occupied(mut entry) => {
                let children = entry.get_mut();
                children.insert(child);
            }
            Entry::Vacant(entry) => {
                self.store(&parent);
                self.init_age_seconds(&parent);
            }
        }
    }

    fn init_age_seconds(&mut self, parent: &Hash) {
        self.tips.get_mut(parent).unwrap().replace(SystemTime::now());
    }

    fn check_num_children_of_parents(&mut self, trunk: &Hash, branch: &Hash) {
        self.check_num_children_of_parent(trunk);
        self.check_num_children_of_parent(branch);
    }

    fn check_num_children_of_parent(&mut self, hash: &Hash) {
        if self.children.get(hash).is_some() {
            if self.num_children(hash) > MAX_NUM_CHILDREN {
                self.remove_tip(&hash);
            }
        }
    }

    fn num_children(&self, hash: &Hash) -> u8 {
        self.children.get(hash).unwrap().len() as u8
    }

    fn remove_tip(&mut self, tip: &Hash) {
        self.tips.remove(&tip);
        self.children.remove(&tip);
        self.non_lazy_tips.remove(&tip);
    }

    // further optimization: avoid allocations
    pub(crate) fn update_scores(&mut self) {

        // reset pool
        self.non_lazy_tips.clear();

        // iter tips and assign them to their appropriate pools
        for (tip, _) in self.tips.clone() {
            match self.tip_score(&tip) {
                Score::NonLazy => {
                    self.non_lazy_tips.insert(tip);
                }
                Score::SemiLazy => {
                    self.tips.remove(&tip);
                    self.children.remove(&tip);
                }
                Score::Lazy => {
                    self.tips.remove(&tip);
                    self.children.remove(&tip);
                }
            }
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
        }

    }

    fn select_tip(&self, hashes: &HashSet<Hash>) -> Option<Hash> {
        if hashes.is_empty() {
            return None;
        }
        Some(*hashes.iter().choose(&mut rand::thread_rng()).unwrap())
    }

}