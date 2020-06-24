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

use bee_crypto::ternary::Hash as TxHash;

use std::collections::HashSet;

/// A Tangle walker that - given a starting vertex - visits all of its ancestors that are connected through
/// the *trunk* edge. The walk continues as long as the visited vertices match a certain condition. For each
/// visited vertex a customized logic can be applied. Each traversed vertex provides read access to its
/// associated data and metadata.
pub fn visit_parents_follow_trunk<'a, Metadata, Match, Apply>(
    tangle: &'a Tangle<Metadata>,
    initial: TxHash,
    matches: Match,
    mut apply: Apply,
) where
    Metadata: Clone + Copy,
    Match: Fn(&TxRef, &Metadata) -> bool,
    Apply: FnMut(&TxHash, &TxRef, &Metadata),
{
    // TODO: how much space is reasonable to preallocate?
    let mut parents = vec![initial];

    while let Some(ref hash) = parents.pop() {
        if let Some(vtx) = tangle.vertices.get(&hash) {
            let vtx = vtx.value();

            if !matches(vtx.transaction(), vtx.metadata()) {
                break;
            } else {
                apply(&hash, vtx.transaction(), vtx.metadata());
                parents.push(*vtx.trunk());
            }
        }
    }
}

/// A Tangle walker that - given a starting vertex - visits all of its children that are connected through
/// the *trunk* edge. The walk continues as long as the visited vertices match a certain condition. For each
/// visited vertex a customized logic can be applied. Each traversed vertex provides read access to its
/// associated data and metadata.
pub fn visit_children_follow_trunk<'a, Metadata, Match, Apply>(
    tangle: &'a Tangle<Metadata>,
    initial: TxHash,
    matches: Match,
    mut apply: Apply,
) where
    Metadata: Clone + Copy,
    Match: Fn(&TxRef, &Metadata) -> bool,
    Apply: FnMut(&TxHash, &TxRef, &Metadata),
{
    let mut children = vec![initial];

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
pub fn visit_parents_depth_first<'a, Metadata, Match, Apply, ElseApply>(
    tangle: &'a Tangle<Metadata>,
    initial: TxHash,
    matches: Match,
    mut apply: Apply,
    mut else_apply: ElseApply,
) where
    Metadata: Clone + Copy,
    Match: Fn(&TxRef, &Metadata) -> bool,
    Apply: FnMut(&TxHash, &TxRef, &Metadata),
    ElseApply: FnMut(&TxHash),
{
    let mut parents = Vec::new();
    let mut visited = HashSet::new();

    parents.push(initial);

    while let Some(hash) = parents.pop() {
        if !visited.contains(&hash) {
            match tangle.vertices.get(&hash) {
                Some(vtx) => {
                    let vtx = vtx.value();

                    apply(&hash, vtx.transaction(), vtx.metadata());

                    if matches(vtx.transaction(), vtx.metadata()) {
                        parents.push(*vtx.trunk());
                        parents.push(*vtx.branch());
                    }
                }
                None => {
                    else_apply(&hash);
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
pub fn visit_children_depth_first<'a, Metadata, Match, Apply, ElseApply>(
    tangle: &'a Tangle<Metadata>,
    initial: TxHash,
    matches: Match,
    mut apply: Apply,
    mut else_apply: ElseApply,
) where
    Metadata: Clone + Copy,
    Match: Fn(&TxRef, &Metadata) -> bool,
    Apply: FnMut(&TxHash, &TxRef, &Metadata),
    ElseApply: FnMut(&TxHash),
{
    let mut children = vec![initial];
    let mut visited = HashSet::new();

    while let Some(hash) = children.last() {
        match tangle.vertices.get(hash) {
            Some(r) => {
                let vtx = r.value();

                if visited.contains(vtx.trunk()) && visited.contains(vtx.branch()) {
                    apply(hash, vtx.transaction(), vtx.metadata());
                    visited.insert(hash.clone());
                    children.pop();
                } else if !visited.contains(vtx.trunk()) {
                    if matches(vtx.transaction(), vtx.metadata()) {
                        children.push(*vtx.trunk());
                    }
                } else if !visited.contains(vtx.branch()) {
                    if matches(vtx.transaction(), vtx.metadata()) {
                        children.push(*vtx.branch());
                    }
                }
            }
            None => {
                // NOTE: this has to be dealt at the protocol level now ;)
                // if !tangle.solid_entry_points.contains(hash) {
                else_apply(hash);
                //}
                visited.insert(hash.clone());
                children.pop();
            }
        }
    }
}
