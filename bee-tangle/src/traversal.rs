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

//! Collection of Tangle traversal functions.

use crate::{tangle::Tangle, TransactionRef as TxRef};

use bee_crypto::ternary::Hash;

use std::collections::HashSet;

/// A Tangle walker that - given a starting vertex - visits all of its ancestors that are connected through
/// the *trunk* edge. The walk continues as long as the visited vertices match a certain condition. For each
/// visited vertex a customized logic can be applied. Each traversed vertex provides read access to its
/// associated data and metadata.
pub fn visit_parents_follow_trunk<Metadata, Match, Apply>(
    tangle: &Tangle<Metadata>,
    mut hash: Hash,
    mut matches: Match,
    mut apply: Apply,
) where
    Metadata: Clone + Copy,
    Match: FnMut(&TxRef, &Metadata) -> bool,
    Apply: FnMut(&Hash, &TxRef, &Metadata),
{
    while let Some(vtx) = tangle.vertices.get(&hash) {
        let vtx = vtx.value();

        if !matches(vtx.transaction(), vtx.metadata()) {
            break;
        } else {
            apply(&hash, vtx.transaction(), vtx.metadata());
            hash = *vtx.trunk();
        }
    }
}

/// A Tangle walker that - given a starting vertex - visits all of its children that are connected through
/// the *trunk* edge. The walk continues as long as the visited vertices match a certain condition. For each
/// visited vertex a customized logic can be applied. Each traversed vertex provides read access to its
/// associated data and metadata.
pub fn visit_children_follow_trunk<Metadata, Match, Apply>(
    tangle: &Tangle<Metadata>,
    root: Hash,
    mut matches: Match,
    mut apply: Apply,
) where
    Metadata: Clone + Copy,
    Match: FnMut(&TxRef, &Metadata) -> bool,
    Apply: FnMut(&Hash, &TxRef, &Metadata),
{
    // TODO could be simplified like visit_parents_follow_trunk ? Meaning no vector ?
    let mut children = vec![root];

    while let Some(ref parent_hash) = children.pop() {
        if let Some(parent) = tangle.vertices.get(parent_hash) {
            if matches(parent.value().transaction(), parent.value().metadata()) {
                apply(parent_hash, parent.value().transaction(), parent.value().metadata());

                if let Some(parent_children) = tangle.children.get(parent_hash) {
                    for child_hash in parent_children.value() {
                        if let Some(child) = tangle.vertices.get(child_hash) {
                            if child.value().trunk() == parent_hash {
                                children.push(*child_hash);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A Tangle walker that - given a starting vertex - visits all of its ancestors that are connected through
/// either the *trunk* or the *branch* edge. The walk continues as long as the visited vertices match a certain
/// condition. For each visited vertex customized logic can be applied depending on the availability of the
/// vertex. Each traversed vertex provides read access to its associated data and metadata.
pub fn visit_parents_depth_first<Metadata, Match, Apply, ElseApply, MissingApply>(
    tangle: &Tangle<Metadata>,
    root: Hash,
    matches: Match,
    mut apply: Apply,
    mut else_apply: ElseApply,
    mut missing_apply: MissingApply,
) where
    Metadata: Clone + Copy,
    Match: Fn(&Hash, &TxRef, &Metadata) -> bool,
    Apply: FnMut(&Hash, &TxRef, &Metadata),
    ElseApply: FnMut(&Hash, &TxRef, &Metadata),
    MissingApply: FnMut(&Hash),
{
    let mut parents = Vec::new();
    let mut visited = HashSet::new();

    parents.push(root);

    while let Some(hash) = parents.pop() {
        if !visited.contains(&hash) {
            match tangle.vertices.get(&hash) {
                Some(vtx) => {
                    let vtx = vtx.value();

                    if matches(&hash, vtx.transaction(), vtx.metadata()) {
                        apply(&hash, vtx.transaction(), vtx.metadata());

                        parents.push(*vtx.trunk());
                        parents.push(*vtx.branch());
                    } else {
                        else_apply(&hash, vtx.transaction(), vtx.metadata());
                    }
                }
                None => {
                    missing_apply(&hash);
                }
            }
            visited.insert(hash);
        }
    }
}

// TODO: test
/// A Tangle walker that - given a starting vertex - visits all of its decendents that are connected through
/// either the *trunk* or the *branch* edge. The walk continues as long as the visited vertices match a certain
/// condition. For each visited vertex customized logic can be applied depending on the availability of the
/// vertex. Each traversed vertex provides read access to its associated data and metadata.
pub fn visit_children_depth_first<Metadata, Match, Apply, ElseApply>(
    tangle: &Tangle<Metadata>,
    root: Hash,
    matches: Match,
    mut apply: Apply,
    mut else_apply: ElseApply,
) where
    Metadata: Clone + Copy,
    Match: Fn(&TxRef, &Metadata) -> bool,
    Apply: FnMut(&Hash, &TxRef, &Metadata),
    ElseApply: FnMut(&Hash),
{
    let mut children = vec![root];
    let mut visited = HashSet::new();

    while let Some(hash) = children.last() {
        match tangle.vertices.get(hash) {
            Some(r) => {
                let vtx = r.value();

                if visited.contains(vtx.trunk()) && visited.contains(vtx.branch()) {
                    apply(hash, vtx.transaction(), vtx.metadata());
                    visited.insert(*hash);
                    children.pop();
                } else if !visited.contains(vtx.trunk()) && matches(vtx.transaction(), vtx.metadata()) {
                    children.push(*vtx.trunk());
                } else if !visited.contains(vtx.branch()) && matches(vtx.transaction(), vtx.metadata()) {
                    children.push(*vtx.branch());
                }
            }
            None => {
                else_apply(hash);
                visited.insert(*hash);
                children.pop();
            }
        }
    }
}
