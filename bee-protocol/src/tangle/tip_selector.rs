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
use bee_transaction::Vertex;
use dashmap::DashSet;
use rand::{seq::IteratorRandom, thread_rng};
use std::collections::HashSet;

enum Score {
    NON_LAZY,
    SEMI_LAZY,
    LAZY,
}

const C1: u32 = 8;
const C2: u32 = 13;
const M: u32 = 15;

const SELECT_TIPS_MAX: u8 = 2;

pub struct TipSelector {
    non_lazy_tips: DashSet<Hash>,
    semi_lazy_tips: DashSet<Hash>,
}

impl TipSelector {
    pub(crate) fn new() -> Self {
        Self {
            non_lazy_tips: DashSet::new(),
            semi_lazy_tips: DashSet::new(),
        }
    }

    pub fn insert(&self, hash: &Hash) {
        self.remove_parents(&hash);
        self.update_scores();
        match self.tip_score(&hash) {
            Score::NON_LAZY => {
                self.non_lazy_tips.insert(hash.clone());
            }
            Score::SEMI_LAZY => {
                self.semi_lazy_tips.insert(hash.clone());
            }
            _ => (),
        }
    }

    // if the parents of this incoming transaction do exist in the pools, remove them
    fn remove_parents(&self, hash: &Hash) {
        let tx = tangle().get(&hash).unwrap();
        if self.non_lazy_tips.contains(tx.trunk()) {
            self.non_lazy_tips.remove(tx.trunk());
        }
        if self.non_lazy_tips.contains(tx.branch()) {
            self.non_lazy_tips.remove(tx.branch());
        }
        if self.semi_lazy_tips.contains(tx.trunk()) {
            self.semi_lazy_tips.remove(tx.trunk());
        }
        if self.semi_lazy_tips.contains(tx.branch()) {
            self.semi_lazy_tips.remove(tx.branch());
        }
    }

    fn update_scores(&self) {
        // check if score changed from non-lazy to semi-lazy or lazy
        for tip in self.non_lazy_tips.clone().iter() {
            match self.tip_score(&tip) {
                Score::SEMI_LAZY => {
                    self.non_lazy_tips.remove(&tip);
                    self.semi_lazy_tips.insert(tip.clone());
                }
                Score::LAZY => {
                    self.non_lazy_tips.remove(&tip);
                }
                _ => (),
            }
        }
        // check if score changed from semi-lazy to lazy
        for tip in self.semi_lazy_tips.clone().iter() {
            match self.tip_score(&tip) {
                Score::LAZY => {
                    self.semi_lazy_tips.remove(&tip);
                }
                _ => (),
            }
        }
    }

    fn remove_max_selected(&self) {
        for tip in self.non_lazy_tips.clone() {
            if tangle().get_metadata(&tip).unwrap().num_selected >= SELECT_TIPS_MAX {
                self.non_lazy_tips.remove(&tip);
            }
        }
        for tip in self.semi_lazy_tips.clone() {
            if tangle().get_metadata(&tip).unwrap().num_selected >= SELECT_TIPS_MAX {
                self.semi_lazy_tips.remove(&tip);
            }
        }
    }

    fn tip_score(&self, hash: &Hash) -> Score {
        let lsmi = *tangle().get_last_solid_milestone_index();
        let otrsi = *tangle().otrsi(&hash).unwrap();
        let ytrsi = *tangle().ytrsi(&hash).unwrap();

        if (lsmi - ytrsi) > C1 {
            return Score::LAZY;
        }

        if (lsmi - otrsi) > C2 {
            return Score::LAZY;
        }

        if (lsmi - otrsi) > M {
            return Score::SEMI_LAZY;
        }

        Score::NON_LAZY
    }

    pub fn get_non_lazy_tips(&self) -> Option<(Hash, Hash)> {
        self.select_tips(&self.non_lazy_tips)
    }

    pub fn get_semi_lazy_tips(&self) -> Option<(Hash, Hash)> {
        self.select_tips(&self.semi_lazy_tips)
    }

    fn select_tips(&self, hashes: &DashSet<Hash>) -> Option<(Hash, Hash)> {
        self.remove_max_selected();
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
}
