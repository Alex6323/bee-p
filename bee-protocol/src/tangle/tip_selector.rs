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
use dashmap::{mapref::entry::Entry, DashMap, DashSet};
use log::{error, info};
use rand::{seq::IteratorRandom, thread_rng};
use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
    time::SystemTime,
};

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
// the maximum time a tip remains in the tip pool after it was referenced by the first transaction. this is used to widen the cone of the tangle. (non-lazy pool)
const MAX_AGE_SECONDS: u8 = 3;
// the maximum amount of children a tip is allowed to have before the tip is removed from the tip pool. this is used to widen the cone of the tangle. (non-lazy pool)
const MAX_NUM_CHILDREN: u8 = 2;
// the maximum count of selection a tip is allowed to get before the tip is removed from the tip pool. this is used to widen the cone of the tangle. (non-lazy pool)
const MAX_NUM_SELECTIONS: u8 = 2;

#[derive(Default)]
pub struct TipSelector {
    hashes: DashMap<Hash, SystemTime>,
    children: DashMap<Hash, HashSet<Hash>>,
    non_lazy_tips: DashSet<Hash>,
    semi_lazy_tips: DashSet<Hash>,
    lock: Arc<RwLock<()>>,
}

impl TipSelector {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    // WURTS only supports solid transactions. This function therefore expects that each passed hash is solid.
    // In context of WURTS, a transaction can be considered as a "tip" if:
    // - transaction is solid and has no children
    // - transaction is solid and has only non-solid children
    // - transaction is solid and has solid children but does not exceed the retention rules
    pub fn insert(&self, hash: &Hash) {
        // store hash
        self.store_hash(hash);
        // link parents with child
        self.add_to_parents(hash);
        // remove parents that have more than 'MAX_CHILDREN_COUNT' children
        self.check_num_children_of_parents(hash);
        // remove hashes that are too old
        self.check_age_seconds();
    }

    fn store_hash(&self, hash: &Hash) {
        self.hashes.insert(*hash, SystemTime::now());
        self.children.insert(*hash, HashSet::new());
    }

    fn add_to_parents(&self, hash: &Hash) {
        let (trunk, branch) = self.parents(hash);
        self.add_child(trunk, *hash);
        self.add_child(branch, *hash);
    }

    fn parents(&self, hash: &Hash) -> (Hash, Hash) {
        let tx = tangle().get(hash).unwrap();
        let trunk = tx.trunk();
        let branch = tx.branch();
        (*trunk, *branch)
    }

    fn add_child(&self, parent: Hash, child: Hash) {
        match self.children.entry(parent) {
            Entry::Occupied(mut entry) => {
                let children = entry.get_mut();
                children.insert(child);
            }
            Entry::Vacant(entry) => {
                let mut children = HashSet::new();
                children.insert(child);
                entry.insert(children);
            }
        }
    }

    fn check_num_children_of_parents(&self, hash: &Hash) {
        let (trunk, branch) = self.parents(hash);
        self.check_num_children_of_parent(&trunk);
        self.check_num_children_of_parent(&branch);
    }

    fn check_num_children_of_parent(&self, hash: &Hash) {
        if self.children.get(hash).is_some() {
            if self.num_children(hash) > MAX_NUM_CHILDREN {
                self.hashes.remove(&hash);
                self.children.remove(&hash);
                self.non_lazy_tips.remove(&hash);
                self.semi_lazy_tips.remove(&hash);
            }
        }
    }

    fn num_children(&self, hash: &Hash) -> u8 {
        self.children.get(hash).unwrap().len() as u8
    }

    fn check_age_seconds(&self) {
        for (tip, time) in self.hashes.clone() {
            match time.elapsed() {
                Ok(elapsed) => {
                    if elapsed.as_secs() as u8 > MAX_AGE_SECONDS {
                        self.hashes.remove(&tip);
                        self.children.remove(&tip);
                    }
                }
                Err(e) => error!("{:?}", e),
            }
        }
    }

    // further optimization: avoid allocations
    pub fn update_scores(&self) {
        // make sure tip selection is not performed while updating the scores
        self.lock.write().unwrap();

        // reset pools
        self.non_lazy_tips.clear();
        self.semi_lazy_tips.clear();

        // iter tips and assign them to their appropriate pools
        for (tip, _) in self.hashes.clone() {
            match self.tip_score(&tip) {
                Score::NonLazy => {
                    self.non_lazy_tips.insert(tip);
                }
                Score::SemiLazy => {
                    self.semi_lazy_tips.insert(tip);
                }
                Score::Lazy => {
                    self.hashes.remove(&tip);
                    self.children.remove(&tip);
                }
            }
        }

        info!(
            "non-lazy {}, semi-lazy {}",
            self.non_lazy_tips.len(),
            self.semi_lazy_tips.len()
        );
    }

    fn tip_score(&self, hash: &Hash) -> Score {
        let lsmi = *tangle().get_last_solid_milestone_index();
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

    pub fn get_semi_lazy_tips(&self) -> Option<(Hash, Hash)> {
        self.select_tips(&self.semi_lazy_tips)
    }

    fn select_tips(&self, hashes: &DashSet<Hash>) -> Option<(Hash, Hash)> {
        // make sure the scores will not get updated during tip selection; optimize to be able to use lock.read()
        // instead, currently needed by num_
        self.lock.write().unwrap();

        self.check_num_selections();
        let mut ret = HashSet::new();

        // try to get 10x randomly a tip
        for i in 1..10 {
            match self.select_tip(hashes) {
                Some(tip) => {
                    ret.insert(tip);
                }
                None => (),
            }
        }
        if ret.is_empty() {
            return None;
        } else if ret.len() == 1 {
            let tip = ret.iter().next().unwrap();
            tangle().update_metadata(&tip, |metadata| {
                metadata.num_selected += 1;
            });
            return Some((*tip, *tip));
        } else {
            let mut iter = ret.iter();
            let tip_1 = *iter.next().unwrap();
            let tip_2 = *iter.next().unwrap();
            tangle().update_metadata(&tip_1, |metadata| {
                metadata.num_selected += 1;
            });
            tangle().update_metadata(&tip_2, |metadata| {
                metadata.num_selected += 1;
            });
            return Some((tip_1, tip_2));
        }
    }

    fn select_tip(&self, hashes: &DashSet<Hash>) -> Option<Hash> {
        if hashes.is_empty() {
            return None;
        }
        Some(*hashes.iter().choose(&mut rand::thread_rng()).unwrap())
    }

    fn check_num_selections(&self) {
        for (hash, _) in self.children.clone() {
            if tangle().is_solid_entry_point(&hash) {
                continue;
            } else {
                if tangle().get_metadata(&hash).unwrap().num_selected >= MAX_NUM_SELECTIONS {
                    self.children.remove(&hash);
                    self.non_lazy_tips.remove(&hash);
                    self.semi_lazy_tips.remove(&hash);
                }
            }
        }
    }
}
