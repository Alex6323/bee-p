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

use crate::{tangle::Tangle, vertex::Vertex, TransactionRef};

use bee_transaction::Hash as THash;

use std::collections::HashSet;

pub fn walk_approvers_post_order_dfs<Meta, Filter, Hit, Miss>(
    tangle: &'static Tangle<Meta>,
    start: THash,
    f: Filter,
    mut h: Hit,
    mut m: Miss,
) where
    Hit: FnMut(&THash, &Vertex<Meta>),
    Filter: Fn(&THash, &Vertex<Meta>) -> bool,
    Miss: FnMut(&THash),
{
    let mut to_visit = vec![start];
    let mut visited = HashSet::new();

    while let Some(hash) = to_visit.last() {
        match tangle.vertices.get(hash) {
            Some(r) => {
                let vtx = r.value();

                if visited.contains(vtx.get_trunk()) && visited.contains(vtx.get_branch()) {
                    h(hash, &vtx);
                    visited.insert(hash.clone());
                    to_visit.pop();
                } else if !visited.contains(vtx.get_trunk()) {
                    if f(hash, &vtx) {
                        to_visit.push(vtx.get_trunk().clone());
                    }
                } else if !visited.contains(vtx.get_branch()) {
                    if f(hash, &vtx) {
                        to_visit.push(vtx.get_branch().clone());
                    }
                }
            }
            None => {
                // NOTE: this has to be dealt at the protocol level now ;)
                // if !tangle.solid_entry_points.contains(hash) {
                m(hash);
                //}
                visited.insert(hash.clone());
                to_visit.pop();
            }
        }
    }
}

pub fn trunk_walk_approvers<Meta, Filter, Action>(tangle: &'static Tangle<Meta>, start: THash, f: Filter, mut a: Action)
where
    Filter: Fn(&Vertex<Meta>) -> bool,
    Action: FnMut(&THash, &Vertex<Meta>),
{
    let mut to_visit = vec![];

    // NOTE: do we need to do this for `start`?
    tangle.vertices.get(&start).map(|r| {
        if f(r.value()) {
            to_visit.push(start);
            a(&start, r.value());
        }
    });

    while let Some(hash) = to_visit.pop() {
        if let Some(r) = tangle.approvers.get(&hash) {
            for approver_hash in r.value() {
                if let Some(s) = tangle.vertices.get(approver_hash) {
                    if s.get_trunk() == &hash && f(s.value()) {
                        to_visit.push(*approver_hash);
                        a(approver_hash, s.value());
                        // NOTE: For simplicity reasons we break here, and assume, that there can't be
                        // a second approver that passes the filter
                        break;
                    }
                }
            }
        }
    }
}

pub fn walk_approvees_dfs<Meta, Follow, Hit, Miss>(
    tangle: &'static Tangle<Meta>,
    start: THash,
    f: Follow,
    mut h: Hit,
    mut m: Miss,
) where
    Follow: Fn(&THash, &Vertex<Meta>) -> bool,
    Hit: FnMut(&THash, &Vertex<Meta>),
    Miss: FnMut(&THash),
{
    let mut to_visit = Vec::new();
    let mut visited = HashSet::new();

    to_visit.push(start);

    while let Some(hash) = to_visit.pop() {
        if !visited.contains(&hash) {
            match tangle.vertices.get(&hash) {
                Some(vtx) => {
                    let vtx = vtx.value();

                    h(&hash, vtx);

                    if f(&hash, vtx) {
                        to_visit.push(*vtx.get_trunk());
                        to_visit.push(*vtx.get_branch());
                    }
                }
                None => {
                    // TODO: need to handle this in protocol
                    // if !self.is_solid_entry_point(&hash) {
                    m(&hash);
                    //}
                }
            }
            visited.insert(hash);
        }
    }
}
