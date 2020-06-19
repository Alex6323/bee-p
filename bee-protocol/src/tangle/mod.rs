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

pub(crate) mod flags;

// TODO: reinstate the async worker
// pub(crate) mod propagator;

use crate::{milestone::MilestoneIndex as MsIndex, tangle::flags::Flags};

use bee_crypto::ternary::Hash as TxHash;
use bee_tangle::{Tangle, TransactionRef as TxRef};
use bee_transaction::{bundled::BundledTransaction as Tx, TransactionVertex};

use dashmap::{DashMap, DashSet};

use std::{
    ptr,
    sync::atomic::{AtomicBool, AtomicPtr, AtomicU32, Ordering},
};

/// Milestone-based Tangle.
pub struct MsTangle {
    pub(crate) inner: Tangle<Flags>,
    pub(crate) milestones: DashMap<MsIndex, TxHash>,
    // TODO use DashMap<TxHash, MilestoneIndex> or DashSet<Sep>, whereby Sep { hash: TxHash, ms: MilestoneIndex }
    pub(crate) solid_entry_points: DashSet<TxHash>,
    solid_milestone_index: AtomicU32,
    last_milestone_index: AtomicU32,
    snapshot_milestone_index: AtomicU32,
}

impl MsTangle {
    pub fn new() -> Self {
        Self {
            inner: Tangle::new(),
            milestones: DashMap::new(),
            solid_entry_points: DashSet::new(),
            solid_milestone_index: AtomicU32::new(0),
            last_milestone_index: AtomicU32::new(0),
            snapshot_milestone_index: AtomicU32::new(0),
        }
    }

    pub fn insert(&self, transaction: Tx, hash: TxHash, flags: Flags) -> Option<TxRef> {
        if let Some(tx) = self.inner.insert(transaction, hash, flags) {
            self.propagate_solid_flag(hash);
            return Some(tx);
        }
        None
    }

    // NOTE: not implemented as an async worker atm, but it makes things much easier
    #[inline]
    fn propagate_solid_flag(&self, initial: TxHash) {
        let mut children = vec![initial];

        while let Some(ref hash) = children.pop() {
            let is_solid = |hash| {
                self.inner
                    .get_metadata(hash)
                    .map(|flags| flags.is_solid())
                    .unwrap_or(false)
            };

            if is_solid(hash) {
                continue;
            }

            if let Some(tx) = self.inner.get(&hash) {
                if is_solid(tx.trunk()) && is_solid(tx.branch()) {
                    self.inner.update_metadata(&hash, |flags| flags.set_solid());

                    for child in self.inner.get_children(&hash) {
                        children.push(child);
                    }
                }
            }
        }
    }

    pub fn get(&self, hash: &TxHash) -> Option<TxRef> {
        self.inner.get(hash)
    }

    pub fn get_flags(&self, hash: &TxHash) -> Option<Flags> {
        self.inner.get_metadata(hash)
    }

    pub fn contains(&self, hash: &TxHash) -> bool {
        self.inner.contains(hash)
    }

    pub fn add_milestone(&self, index: MsIndex, hash: TxHash) {
        // TODO: only insert if vacant
        self.milestones.insert(index, hash);

        if let Some(mut metadata) = self.inner.get_metadata(&hash) {
            metadata.set_milestone();

            self.inner.set_metadata(&hash, metadata);
        }
    }

    pub fn remove_milestone(&self, index: MsIndex) {
        self.milestones.remove(&index);
    }

    // TODO: use combinator instead of match
    pub fn get_milestone(&self, index: MsIndex) -> Option<TxRef> {
        match self.get_milestone_hash(index) {
            None => None,
            Some(ref hash) => self.get(hash),
        }
    }

    // TODO: use combinator instead of match
    pub fn get_milestone_hash(&self, index: MsIndex) -> Option<TxHash> {
        match self.milestones.get(&index) {
            None => None,
            Some(v) => Some(*v),
        }
    }

    pub fn contains_milestone(&self, index: MsIndex) -> bool {
        self.milestones.contains_key(&index)
    }

    pub fn get_solid_milestone_index(&self) -> MsIndex {
        self.solid_milestone_index.load(Ordering::Relaxed).into()
    }

    pub fn get_last_milestone_index(&self) -> MsIndex {
        self.last_milestone_index.load(Ordering::Relaxed).into()
    }

    pub fn get_snapshot_milestone_index(&self) -> MsIndex {
        self.snapshot_milestone_index.load(Ordering::Relaxed).into()
    }

    pub fn update_solid_milestone_index(&self, new_index: MsIndex) {
        self.solid_milestone_index.store(*new_index, Ordering::Relaxed);
    }

    pub fn update_last_milestone_index(&self, new_index: MsIndex) {
        self.last_milestone_index.store(*new_index, Ordering::Relaxed);
    }

    pub fn update_snapshot_milestone_index(&self, new_index: MsIndex) {
        self.snapshot_milestone_index.store(*new_index, Ordering::Relaxed);
    }

    pub fn is_synced(&self) -> bool {
        self.get_solid_milestone_index() == self.get_last_milestone_index()
    }

    pub fn add_solid_entry_point(&self, hash: TxHash) {
        self.solid_entry_points.insert(hash);
    }

    /// Removes `hash` from the set of solid entry points.
    pub fn remove_solid_entry_point(&self, hash: &TxHash) {
        self.solid_entry_points.remove(hash);
    }

    /// Returns whether the transaction associated with `hash` is a solid entry point.
    pub fn is_solid_entry_point(&self, hash: &TxHash) -> bool {
        self.solid_entry_points.contains(hash)
    }

    /// Returns whether the transaction associated with `hash` is deemed `solid`.
    pub fn is_solid_transaction(&self, hash: &TxHash) -> bool {
        if self.is_solid_entry_point(hash) {
            true
        } else {
            self.inner.get_metadata(hash).map(|m| m.is_solid()).unwrap_or(false)
        }
    }

    /// Returns a reference to the inner (abstract) Tangle.
    pub fn inner(&self) -> &Tangle<Flags> {
        &self.inner
    }

    /// Returns the size of the current Tangle.
    pub fn size(&self) -> usize {
        self.inner.size()
    }
}

static TANGLE: AtomicPtr<MsTangle> = AtomicPtr::new(ptr::null_mut());
static INITIALIZED: AtomicBool = AtomicBool::new(false);

pub fn init() {
    if !INITIALIZED.compare_and_swap(false, true, Ordering::Relaxed) {
        TANGLE.store(Box::into_raw(MsTangle::new().into()), Ordering::Relaxed);
    } else {
        panic!("Tangle already initialized");
    }
}

pub fn tangle() -> &'static MsTangle {
    let tangle = TANGLE.load(Ordering::Relaxed);
    if tangle.is_null() {
        panic!("Tangle cannot be null");
    } else {
        unsafe { &*tangle }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tangle::Flags;

    use bee_tangle::traversal;
    use bee_test::{field::rand_trits_field, transaction::create_random_attached_tx};
    use bee_transaction::Hash as TxHash;

    #[test]
    fn confirm_transaction() {
        // Example from https://github.com/iotaledger/protocol-rfcs/blob/master/text/0005-white-flag/0005-white-flag.md

        let tangle = MsTangle::new();

        // Creates solid entry points
        let sep1 = rand_trits_field::<TxHash>();
        let sep2 = rand_trits_field::<TxHash>();
        let sep3 = rand_trits_field::<TxHash>();
        let sep4 = rand_trits_field::<TxHash>();
        let sep5 = rand_trits_field::<TxHash>();
        let sep6 = rand_trits_field::<TxHash>();

        // Adds solid entry points
        tangle.add_solid_entry_point(sep1);
        tangle.add_solid_entry_point(sep2);
        tangle.add_solid_entry_point(sep3);
        tangle.add_solid_entry_point(sep4);
        tangle.add_solid_entry_point(sep5);
        tangle.add_solid_entry_point(sep6);

        // Links transactions
        let (a_hash, a) = create_random_attached_tx(sep1, sep2);
        let (b_hash, b) = create_random_attached_tx(sep3, sep4);
        let (c_hash, c) = create_random_attached_tx(sep5, sep6);
        let (d_hash, d) = create_random_attached_tx(b_hash, a_hash);
        let (e_hash, e) = create_random_attached_tx(b_hash, a_hash);
        let (f_hash, f) = create_random_attached_tx(c_hash, b_hash);
        let (g_hash, g) = create_random_attached_tx(e_hash, d_hash);
        let (h_hash, h) = create_random_attached_tx(f_hash, e_hash);
        let (i_hash, i) = create_random_attached_tx(c_hash, f_hash);
        let (j_hash, j) = create_random_attached_tx(h_hash, g_hash);
        let (k_hash, k) = create_random_attached_tx(i_hash, h_hash);
        let (l_hash, l) = create_random_attached_tx(j_hash, g_hash);
        let (m_hash, m) = create_random_attached_tx(h_hash, j_hash);
        let (n_hash, n) = create_random_attached_tx(k_hash, h_hash);
        let (o_hash, o) = create_random_attached_tx(i_hash, k_hash);
        let (p_hash, p) = create_random_attached_tx(i_hash, k_hash);
        let (q_hash, q) = create_random_attached_tx(m_hash, l_hash);
        let (r_hash, r) = create_random_attached_tx(m_hash, l_hash);
        let (s_hash, s) = create_random_attached_tx(o_hash, n_hash);
        let (t_hash, t) = create_random_attached_tx(p_hash, o_hash);
        let (u_hash, u) = create_random_attached_tx(r_hash, q_hash);
        let (v_hash, v) = create_random_attached_tx(s_hash, r_hash);
        let (w_hash, w) = create_random_attached_tx(t_hash, s_hash);
        let (x_hash, x) = create_random_attached_tx(u_hash, q_hash);
        let (y_hash, y) = create_random_attached_tx(v_hash, u_hash);
        let (z_hash, z) = create_random_attached_tx(s_hash, v_hash);

        // Confirms transactions
        // TODO uncomment when confirmation index
        // tangle.confirm_transaction(a_hash, 1);
        // tangle.confirm_transaction(b_hash, 1);
        // tangle.confirm_transaction(c_hash, 1);
        // tangle.confirm_transaction(d_hash, 2);
        // tangle.confirm_transaction(e_hash, 1);
        // tangle.confirm_transaction(f_hash, 1);
        // tangle.confirm_transaction(g_hash, 2);
        // tangle.confirm_transaction(h_hash, 1);
        // tangle.confirm_transaction(i_hash, 2);
        // tangle.confirm_transaction(j_hash, 2);
        // tangle.confirm_transaction(k_hash, 2);
        // tangle.confirm_transaction(l_hash, 2);
        // tangle.confirm_transaction(m_hash, 2);
        // tangle.confirm_transaction(n_hash, 2);
        // tangle.confirm_transaction(o_hash, 2);
        // tangle.confirm_transaction(p_hash, 3);
        // tangle.confirm_transaction(q_hash, 3);
        // tangle.confirm_transaction(r_hash, 2);
        // tangle.confirm_transaction(s_hash, 2);
        // tangle.confirm_transaction(t_hash, 3);
        // tangle.confirm_transaction(u_hash, 3);
        // tangle.confirm_transaction(v_hash, 2);
        // tangle.confirm_transaction(w_hash, 3);
        // tangle.confirm_transaction(x_hash, 3);
        // tangle.confirm_transaction(y_hash, 3);
        // tangle.confirm_transaction(z_hash, 3);

        // Constructs the graph
        tangle.insert(a, a_hash, Flags::empty());
        tangle.insert(b, b_hash, Flags::empty());
        tangle.insert(c, c_hash, Flags::empty());
        tangle.insert(d, d_hash, Flags::empty());
        tangle.insert(e, e_hash, Flags::empty());
        tangle.insert(f, f_hash, Flags::empty());
        tangle.insert(g, g_hash, Flags::empty());
        tangle.insert(h, h_hash, Flags::empty());
        tangle.insert(i, i_hash, Flags::empty());
        tangle.insert(j, j_hash, Flags::empty());
        tangle.insert(k, k_hash, Flags::empty());
        tangle.insert(l, l_hash, Flags::empty());
        tangle.insert(m, m_hash, Flags::empty());
        tangle.insert(n, n_hash, Flags::empty());
        tangle.insert(o, o_hash, Flags::empty());
        tangle.insert(p, p_hash, Flags::empty());
        tangle.insert(q, q_hash, Flags::empty());
        tangle.insert(r, r_hash, Flags::empty());
        tangle.insert(s, s_hash, Flags::empty());
        tangle.insert(t, t_hash, Flags::empty());
        tangle.insert(u, u_hash, Flags::empty());
        tangle.insert(v, v_hash, Flags::empty());
        tangle.insert(w, w_hash, Flags::empty());
        tangle.insert(x, x_hash, Flags::empty());
        tangle.insert(y, y_hash, Flags::empty());
        tangle.insert(z, z_hash, Flags::empty());

        let mut hashes = Vec::new();

        traversal::visit_children_depth_first(
            &tangle.inner,
            v_hash,
            |_, _| true,
            |hash, _tx, _metadata| hashes.push(*hash),
            |_| (),
        );

        // TODO Remove when we have confirmation index
        assert_eq!(hashes.len(), 18);

        assert_eq!(hashes[0], a_hash);
        assert_eq!(hashes[1], b_hash);
        assert_eq!(hashes[2], d_hash);
        assert_eq!(hashes[3], e_hash);
        assert_eq!(hashes[4], g_hash);
        assert_eq!(hashes[5], c_hash);
        assert_eq!(hashes[6], f_hash);
        assert_eq!(hashes[7], h_hash);
        assert_eq!(hashes[8], j_hash);
        assert_eq!(hashes[9], l_hash);
        assert_eq!(hashes[10], m_hash);
        assert_eq!(hashes[11], r_hash);
        assert_eq!(hashes[12], i_hash);
        assert_eq!(hashes[13], k_hash);
        assert_eq!(hashes[14], n_hash);
        assert_eq!(hashes[15], o_hash);
        assert_eq!(hashes[16], s_hash);
        assert_eq!(hashes[17], v_hash);

        // TODO uncomment when we have confirmation index
        // assert_eq!(hashes.len(), 12);
        // assert_eq!(hashes[0], d_hash);
        // assert_eq!(hashes[1], g_hash);
        // assert_eq!(hashes[2], j_hash);
        // assert_eq!(hashes[3], l_hash);
        // assert_eq!(hashes[4], m_hash);
        // assert_eq!(hashes[5], r_hash);
        // assert_eq!(hashes[6], i_hash);
        // assert_eq!(hashes[7], k_hash);
        // assert_eq!(hashes[8], n_hash);
        // assert_eq!(hashes[9], o_hash);
        // assert_eq!(hashes[10], s_hash);
        // assert_eq!(hashes[11], v_hash);
    }
}

// use crate::{
//     milestone::MilestoneIndex,
//     vertex::{TransactionRef, Vertex},
// };

// use bee_bundle::{Hash, Transaction};

// use std::{
//     collections::HashSet,
//     sync::atomic::{AtomicU32, Ordering},
// };

// use async_std::{
//     sync::{Arc, Barrier},
//     task::block_on,
// };

// use dashmap::{mapref::entry::Entry, DashMap, DashSet};

// use flume::Sender;

// /// A datastructure based on a directed acyclic graph (DAG).
// pub struct Tangle<T> {
//     /// A map between each vertex and the hash of the transaction the respective vertex represents.
//     pub(crate) vertices: DashMap<Hash, Vertex<T>>,

//     /// A map between the hash of a transaction and the hashes of its approvers.
//     pub(crate) approvers: DashMap<Hash, Vec<Hash>>,

//     /// A map between the milestone index and hash of the milestone transaction.
//     milestones: DashMap<MilestoneIndex, Hash>,

//     /// A set of hashes representing transactions deemed solid entry points.
//     solid_entry_points: DashSet<Hash>,

//     /// The sender side of a channel between the Tangle and the (gossip) solidifier.
//     solidifier_send: Sender<Option<Hash>>,

//     solid_milestone_index: AtomicU32,
//     snapshot_milestone_index: AtomicU32,
//     last_milestone_index: AtomicU32,

//     drop_barrier: Arc<Barrier>,
// }

// impl<T> Tangle<T> {
//     /// Creates a new `Tangle`.
//     pub(crate) fn new(solidifier_send: Sender<Option<Hash>>, drop_barrier: Arc<Barrier>) -> Self {
//         Self {
//             vertices: DashMap::new(),
//             approvers: DashMap::new(),
//             solidifier_send,
//             solid_entry_points: DashSet::new(),
//             milestones: DashMap::new(),
//             solid_milestone_index: AtomicU32::new(0),
//             snapshot_milestone_index: AtomicU32::new(0),
//             last_milestone_index: AtomicU32::new(0),
//             drop_barrier,
//         }
//     }

//     /// Inserts a transaction.
//     ///
//     /// Note: The method assumes that `hash` -> `transaction` is injective, otherwise unexpected behavior could
//     /// occur.
//     pub async fn insert_transaction(
//         &'static self,
//         transaction: Transaction,
//         hash: Hash,
//         meta: T,
//     ) -> Option<TransactionRef> {
//         match self.approvers.entry(*transaction.trunk()) {
//             Entry::Occupied(mut entry) => {
//                 let values = entry.get_mut();
//                 values.push(hash);
//             }
//             Entry::Vacant(entry) => {
//                 entry.insert(vec![hash]);
//             }
//         }

//         if transaction.trunk() != transaction.branch() {
//             match self.approvers.entry(*transaction.branch()) {
//                 Entry::Occupied(mut entry) => {
//                     let values = entry.get_mut();
//                     values.push(hash);
//                 }
//                 Entry::Vacant(entry) => {
//                     entry.insert(vec![hash]);
//                 }
//             }
//         }

//         let vertex = Vertex::from(transaction, hash, meta);

//         let tx_ref = vertex.get_ref_to_inner();

//         // TODO: not sure if we want replacement of vertices
//         if self.vertices.insert(hash, vertex).is_none() {
//             match self.solidifier_send.send(Some(hash)) {
//                 Ok(()) => (),
//                 Err(e) => todo!("log warning"),
//             }

//             Some(tx_ref)
//         } else {
//             None
//         }
//     }

//     pub(crate) fn shutdown(&self) {
//         // `None` will cause the worker to finish
//         self.solidifier_send.send(None).expect("error sending shutdown signal");
//         block_on(self.drop_barrier.wait());
//     }

//     /// Returns a reference to a transaction, if it's available in the local Tangle.
//     pub fn get_transaction(&'static self, hash: &Hash) -> Option<TransactionRef> {
//         self.vertices.get(hash).map(|v| v.get_ref_to_inner())
//     }

//     /// Returns whether the transaction is stored in the Tangle.
//     pub fn contains_transaction(&'static self, hash: &Hash) -> bool {
//         self.vertices.contains_key(hash)
//     }

//     /// Returns whether the transaction associated with `hash` is solid.
//     ///
//     /// Note: This function is _eventually consistent_ - if `true` is returned, solidification has
//     /// definitely occurred. If `false` is returned, then solidification has probably not occurred,
//     /// or solidification information has not yet been fully propagated.
//     pub fn is_solid_transaction(&'static self, hash: &Hash) -> bool {
//         if self.is_solid_entry_point(hash) {
//             true
//         } else {
//             self.vertices.get(hash).map(|r| r.value().is_solid()).unwrap_or(false)
//         }
//     }

//     /// Adds the `hash` of a milestone identified by its milestone `index`.
//     pub fn add_milestone(&'static self, index: MilestoneIndex, hash: Hash) {
//         self.milestones.insert(index, hash);
//         if let Some(mut vertex) = self.vertices.get_mut(&hash) {
//             vertex.set_milestone();
//         }
//     }

//     /// Removes the hash of a milestone.
//     pub fn remove_milestone(&'static self, index: MilestoneIndex) {
//         self.milestones.remove(&index);
//     }

//     /// Returns the milestone transaction corresponding to the given milestone `index`.
//     pub fn get_milestone(&'static self, index: MilestoneIndex) -> Option<TransactionRef> {
//         match self.get_milestone_hash(index) {
//             None => None,
//             Some(hash) => self.get_transaction(&hash),
//         }
//     }

//     /// Returns a [`VertexRef`] linked to the specified milestone, if it's available in the local Tangle.
//     pub fn get_latest_milestone(&'static self) -> Option<TransactionRef> {
//         todo!("get the last milestone index, get the transaction hash from it, and query the Tangle for it")
//     }

//     /// Returns the hash of a milestone.
//     pub fn get_milestone_hash(&'static self, index: MilestoneIndex) -> Option<Hash> {
//         match self.milestones.get(&index) {
//             None => None,
//             Some(v) => Some(*v),
//         }
//     }

//     /// Returns whether the milestone index maps to a know milestone hash.
//     pub fn contains_milestone(&'static self, index: MilestoneIndex) -> bool {
//         self.milestones.contains_key(&index)
//     }

//     /// Retreives the solid milestone index.
//     pub fn get_solid_milestone_index(&'static self) -> MilestoneIndex {
//         self.solid_milestone_index.load(Ordering::Relaxed).into()
//     }

//     /// Updates the solid milestone index to `new_index`.
//     pub fn update_solid_milestone_index(&'static self, new_index: MilestoneIndex) {
//         self.solid_milestone_index.store(*new_index, Ordering::Relaxed);
//     }

//     /// Retreives the snapshot milestone index.
//     pub fn get_snapshot_milestone_index(&'static self) -> MilestoneIndex {
//         self.snapshot_milestone_index.load(Ordering::Relaxed).into()
//     }

//     /// Updates the snapshot milestone index to `new_index`.
//     pub fn update_snapshot_milestone_index(&'static self, new_index: MilestoneIndex) {
//         self.snapshot_milestone_index.store(*new_index, Ordering::Relaxed);
//     }

//     /// Retreives the last milestone index.
//     pub fn get_last_milestone_index(&'static self) -> MilestoneIndex {
//         self.last_milestone_index.load(Ordering::Relaxed).into()
//     }

//     /// Updates the last milestone index to `new_index`.
//     pub fn update_last_milestone_index(&'static self, new_index: MilestoneIndex) {
//         self.last_milestone_index.store(*new_index, Ordering::Relaxed);
//     }

//     /// Adds `hash` to the set of solid entry points.
//     pub fn add_solid_entry_point(&'static self, hash: Hash) {
//         self.solid_entry_points.insert(hash);
//     }

//     /// Removes `hash` from the set of solid entry points.
//     pub fn remove_solid_entry_point(&'static self, hash: Hash) {
//         self.solid_entry_points.remove(&hash);
//     }

//     /// Returns whether the transaction associated `hash` is a solid entry point.
//     pub fn is_solid_entry_point(&'static self, hash: &Hash) -> bool {
//         self.solid_entry_points.contains(hash)
//     }

//     /// Checks if the tangle is synced or not
//     pub fn is_synced(&'static self) -> bool {
//         self.get_solid_milestone_index() == self.get_last_milestone_index()
//     }

//     /// Returns the current size of the Tangle.
//     pub fn size(&'static self) -> usize {
//         self.vertices.len()
//     }

//     /// Starts a walk beginning at a `start` vertex identified by its associated transaction hash
//     /// traversing its children/approvers for as long as those satisfy a given `filter`.
//     ///
//     /// Returns a list of descendents of `start`. It is ensured, that all elements of that list
//     /// are connected through the trunk.
//     pub fn trunk_walk_approvers<F>(&'static self, start: Hash, filter: F) -> Vec<(TransactionRef, Hash)>
//     where
//         F: Fn(&TransactionRef) -> bool,
//     {
//         let mut approvees = vec![];
//         let mut collected = vec![];

//         if let Some(approvee_ref) = self.vertices.get(&start) {
//             let approvee_vtx = approvee_ref.value();
//             let approvee = approvee_vtx.get_ref_to_inner();

//             if filter(&approvee) {
//                 approvees.push(start);
//                 collected.push((approvee, approvee_vtx.get_id()));

//                 while let Some(approvee_hash) = approvees.pop() {
//                     if let Some(approvers_ref) = self.approvers.get(&approvee_hash) {
//                         for approver_hash in approvers_ref.value() {
//                             if let Some(approver_ref) = self.vertices.get(approver_hash) {
//                                 let approver = approver_ref.value().get_ref_to_inner();

//                                 if *approver.trunk() == approvee_hash && filter(&approver) {
//                                     approvees.push(*approver_hash);
//                                     collected.push((approver, approver_ref.value().get_id()));
//                                     // NOTE: For simplicity reasons we break here, and assume, that there can't be
//                                     // a second approver that passes the filter
//                                     break;
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }
//         }

//         collected
//     }

//     /// Starts a walk beginning at a `start` vertex identified by its associated transaction hash
//     /// traversing its ancestors/approvees for as long as those satisfy a given `filter`.
//     ///
//     /// Returns a list of ancestors of `start`. It is ensured, that all elements of that list
//     /// are connected through the trunk.
//     pub fn trunk_walk_approvees<F>(&'static self, start: Hash, filter: F) -> Vec<(TransactionRef, Hash)>
//     where
//         F: Fn(&TransactionRef) -> bool,
//     {
//         let mut approvers = vec![start];
//         let mut collected = vec![];

//         while let Some(approver_hash) = approvers.pop() {
//             if let Some(approver_ref) = self.vertices.get(&approver_hash) {
//                 let approver_vtx = approver_ref.value();
//                 let approver = approver_vtx.get_ref_to_inner();

//                 if !filter(&approver) {
//                     break;
//                 } else {
//                     approvers.push(approver.trunk().clone());
//                     collected.push((approver, approver_vtx.get_id()));
//                 }
//             }
//         }

//         collected
//     }

//     /// Walks all approvers given a starting hash `root`.
//     pub fn walk_approvees_depth_first<Mapping, Follow, Missing>(
//         &'static self,
//         root: Hash,
//         mut map: Mapping,
//         should_follow: Follow,
//         mut on_missing: Missing,
//     ) where
//         Mapping: FnMut(&TransactionRef),
//         Follow: Fn(&Vertex<T>) -> bool,
//         Missing: FnMut(&Hash),
//     {
//         let mut non_analyzed_hashes = Vec::new();
//         let mut analyzed_hashes = HashSet::new();

//         non_analyzed_hashes.push(root);

//         while let Some(hash) = non_analyzed_hashes.pop() {
//             if !analyzed_hashes.contains(&hash) {
//                 match self.vertices.get(&hash) {
//                     Some(vertex) => {
//                         let vertex = vertex.value();
//                         let transaction = vertex.get_ref_to_inner();

//                         map(&transaction);

//                         if should_follow(vertex) {
//                             non_analyzed_hashes.push(*transaction.branch());
//                             non_analyzed_hashes.push(*transaction.trunk());
//                         }
//                     }
//                     None => {
//                         if !self.is_solid_entry_point(&hash) {
//                             on_missing(&hash);
//                         }
//                     }
//                 }
//                 analyzed_hashes.insert(hash);
//             }
//         }
//     }

//     /// Walks all approvers in a post order DFS way through trunk then branch.
//     pub fn walk_approvers_post_order_dfs<Mapping, Follow, Missing>(
//         &'static self,
//         root: Hash,
//         mut map: Mapping,
//         should_follow: Follow,
//         mut on_missing: Missing,
//     ) where
//         Mapping: FnMut(&Hash, &TransactionRef),
//         Follow: Fn(&Vertex<T>) -> bool,
//         Missing: FnMut(&Hash),
//     {
//         let mut non_analyzed_hashes = Vec::new();
//         let mut analyzed_hashes = HashSet::new();

//         non_analyzed_hashes.push(root);

//         while let Some(hash) = non_analyzed_hashes.last() {
//             match self.vertices.get(hash) {
//                 Some(vertex) => {
//                     let vertex = vertex.value();
//                     let transaction = vertex.get_ref_to_inner();

//                     // TODO add follow
//                     if analyzed_hashes.contains(transaction.trunk()) &&
// analyzed_hashes.contains(transaction.branch()) {                         map(hash, &transaction);
//                         analyzed_hashes.insert(hash.clone());
//                         non_analyzed_hashes.pop();
//                     // TODO add follow
//                     } else if !analyzed_hashes.contains(transaction.trunk()) {
//                         non_analyzed_hashes.push(*transaction.trunk());
//                     // TODO add follow
//                     } else if !analyzed_hashes.contains(transaction.branch()) {
//                         non_analyzed_hashes.push(*transaction.branch());
//                     }
//                 }
//                 None => {
//                     if !self.is_solid_entry_point(hash) {
//                         on_missing(hash);
//                     }
//                     analyzed_hashes.insert(hash.clone());
//                     non_analyzed_hashes.pop();
//                 }
//             }
//         }
//     }

//     #[cfg(test)]
//     fn num_approvers(&'static self, hash: &Hash) -> usize {
//         self.approvers.get(hash).map_or(0, |r| r.value().len())
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::*;

//     #[test]
//     #[serial]
//     fn update_and_get_snapshot_milestone_index() {
//         init();
//         let tangle = tangle();

//         tangle.update_snapshot_milestone_index(1368160.into());

//         assert_eq!(1368160, *tangle.get_snapshot_milestone_index());
//         drop();
//     }

//     #[test]
//     #[serial]
//     fn update_and_get_solid_milestone_index() {
//         init();
//         let tangle = tangle();

//         tangle.update_solid_milestone_index(1368167.into());

//         assert_eq!(1368167, *tangle.get_solid_milestone_index());
//         drop();
//     }

//     #[test]
//     #[serial]
//     fn update_and_get_last_milestone_index() {
//         init();
//         let tangle = tangle();

//         tangle.update_last_milestone_index(1368168.into());

//         assert_eq!(1368168, *tangle.get_last_milestone_index());
//         drop();
//     }

// ----

// pub use milestone::MilestoneIndex;
// pub use tangle::Tangle;
// pub use vertex::TransactionRef;

// //mod milestone;
// //mod solidifier;
// mod tangle;
// mod vertex;

// use solidifier::SolidifierState;

// use async_std::{
//     sync::{channel, Arc, Barrier},
//     task::spawn,
// };

// use bee_bundle::Hash;

// use std::{
//     ptr,
//     sync::atomic::{AtomicBool, AtomicPtr, Ordering},
// };

// static TANGLE: AtomicPtr<Tangle<u8>> = AtomicPtr::new(ptr::null_mut());
// static INITIALIZED: AtomicBool = AtomicBool::new(false);

// const SOLIDIFIER_CHAN_CAPACITY: usize = 1000;

// /// Initializes the Tangle singleton.
// pub fn init() {
//     if !INITIALIZED.compare_and_swap(false, true, Ordering::Relaxed) {
//         let (sender, receiver) = flume::bounded::<Option<Hash>>(SOLIDIFIER_CHAN_CAPACITY);

//         let drop_barrier = async_std::sync::Arc::new(Barrier::new(2));

//         TANGLE.store(
//             Box::into_raw(Tangle::new(sender, drop_barrier.clone()).into()),
//             Ordering::Relaxed,
//         );

//         spawn(SolidifierState::new(receiver, drop_barrier).run());
//     } else {
//         drop();
//         panic!("Already initialized");
//     }
// }

// /// Returns the singleton instance of the Tangle.
// pub fn tangle() -> &'static Tangle<u8> {
//     let tangle = TANGLE.load(Ordering::Relaxed);
//     if tangle.is_null() {
//         panic!("Tangle cannot be null");
//     } else {
//         unsafe { &*tangle }
//     }
// }

// /// Drops the Tangle singleton.
// pub fn drop() {
//     if INITIALIZED.compare_and_swap(true, false, Ordering::Relaxed) {
//         tangle().shutdown();

//         let tangle = TANGLE.swap(ptr::null_mut(), Ordering::Relaxed);
//         if !tangle.is_null() {
//             let _ = unsafe { Box::from_raw(tangle) };
//         }
//     } else {
//         panic!("Already dropped");
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use serial_test::serial;

//     #[test]
//     #[serial]
//     fn init_get_and_drop() {
//         init();
//         let _ = tangle();
//         drop();
//     }

//     #[test]
//     #[should_panic]
//     #[serial]
//     fn double_init_should_panic() {
//         init();
//         init();
//     }

//     #[test]
//     #[should_panic]
//     #[serial]
//     fn double_drop_should_panic() {
//         init();
//         drop();
//         drop();
//     }

//     #[test]
//     #[should_panic]
//     #[serial]
//     fn drop_without_init_should_panic() {
//         drop();
//     }

//     #[test]
//     #[should_panic]
//     #[serial]
//     fn get_without_init_should_panic() {
//         let _ = tangle();
//         drop();
//     }
// }
