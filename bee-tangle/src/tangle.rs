//! Module that provides the [`Tangle`] struct.

use crate::{
    milestone::MilestoneIndex,
    vertex::{TransactionRef, Vertex},
};

use bee_bundle::{Hash, Transaction};

use std::{
    collections::HashSet,
    sync::atomic::{AtomicU32, Ordering},
};

use async_std::{
    sync::{Arc, Barrier},
    task::block_on,
};

use dashmap::{mapref::entry::Entry, DashMap, DashSet};

use flume::Sender;

/// A datastructure based on a directed acyclic graph (DAG).
pub struct Tangle {
    /// A map between each vertex and the hash of the transaction the respective vertex represents.
    pub(crate) vertices: DashMap<Hash, Vertex>,

    /// A map between the hash of a transaction and the hashes of its approvers.
    pub(crate) approvers: DashMap<Hash, Vec<Hash>>,

    /// A map between the milestone index and hash of the milestone transaction.
    milestones: DashMap<MilestoneIndex, Hash>,

    /// A set of hashes representing transactions deemed solid entry points.
    solid_entry_points: DashSet<Hash>,

    /// The sender side of a channel between the Tangle and the (gossip) solidifier.
    solidifier_send: Sender<Option<Hash>>,

    solid_milestone_index: AtomicU32,
    snapshot_milestone_index: AtomicU32,
    last_milestone_index: AtomicU32,

    drop_barrier: Arc<Barrier>,
}

impl Tangle {
    /// Creates a new `Tangle`.
    pub(crate) fn new(solidifier_send: Sender<Option<Hash>>, drop_barrier: Arc<Barrier>) -> Self {
        Self {
            vertices: DashMap::new(),
            approvers: DashMap::new(),
            solidifier_send,
            solid_entry_points: DashSet::new(),
            milestones: DashMap::new(),
            solid_milestone_index: AtomicU32::new(0),
            snapshot_milestone_index: AtomicU32::new(0),
            last_milestone_index: AtomicU32::new(0),
            drop_barrier,
        }
    }

    /// Inserts a transaction.
    ///
    /// Note: The method assumes that `hash` -> `transaction` is injective, otherwise unexpected behavior could
    /// occur.
    pub async fn insert_transaction(&'static self, transaction: Transaction, hash: Hash) -> Option<TransactionRef> {
        match self.approvers.entry(*transaction.trunk()) {
            Entry::Occupied(mut entry) => {
                let values = entry.get_mut();
                values.push(hash);
            }
            Entry::Vacant(entry) => {
                entry.insert(vec![hash]);
            }
        }

        if transaction.trunk() != transaction.branch() {
            match self.approvers.entry(*transaction.branch()) {
                Entry::Occupied(mut entry) => {
                    let values = entry.get_mut();
                    values.push(hash);
                }
                Entry::Vacant(entry) => {
                    entry.insert(vec![hash]);
                }
            }
        }

        let vertex = Vertex::from(transaction, hash);

        let tx_ref = vertex.get_ref_to_inner();

        // TODO: not sure if we want replacement of vertices
        if self.vertices.insert(hash, vertex).is_none() {
            match self.solidifier_send.send(Some(hash)) {
                Ok(()) => (),
                Err(e) => todo!("log warning"),
            }

            Some(tx_ref)
        } else {
            None
        }
    }

    pub(crate) fn shutdown(&self) {
        // `None` will cause the worker to finish
        self.solidifier_send.send(None).expect("error sending shutdown signal");
        block_on(self.drop_barrier.wait());
    }

    /// Returns a reference to a transaction, if it's available in the local Tangle.
    pub fn get_transaction(&'static self, hash: &Hash) -> Option<TransactionRef> {
        self.vertices.get(hash).map(|v| v.get_ref_to_inner())
    }

    /// Returns whether the transaction is stored in the Tangle.
    pub fn contains_transaction(&'static self, hash: &Hash) -> bool {
        self.vertices.contains_key(hash)
    }

    /// Returns whether the transaction associated with `hash` is solid.
    ///
    /// Note: This function is _eventually consistent_ - if `true` is returned, solidification has
    /// definitely occurred. If `false` is returned, then solidification has probably not occurred,
    /// or solidification information has not yet been fully propagated.
    pub fn is_solid_transaction(&'static self, hash: &Hash) -> bool {
        if self.is_solid_entry_point(hash) {
            true
        } else {
            self.vertices.get(hash).map(|r| r.value().is_solid()).unwrap_or(false)
        }
    }

    /// Adds the `hash` of a milestone identified by its milestone `index`.
    pub fn add_milestone(&'static self, index: MilestoneIndex, hash: Hash) {
        self.milestones.insert(index, hash);
        if let Some(mut vertex) = self.vertices.get_mut(&hash) {
            vertex.set_milestone();
        }
    }

    /// Removes the hash of a milestone.
    pub fn remove_milestone(&'static self, index: MilestoneIndex) {
        self.milestones.remove(&index);
    }

    /// Returns the milestone transaction corresponding to the given milestone `index`.
    pub fn get_milestone(&'static self, index: MilestoneIndex) -> Option<TransactionRef> {
        match self.get_milestone_hash(index) {
            None => None,
            Some(hash) => self.get_transaction(&hash),
        }
    }

    /// Returns a [`VertexRef`] linked to the specified milestone, if it's available in the local Tangle.
    pub fn get_latest_milestone(&'static self) -> Option<TransactionRef> {
        todo!("get the last milestone index, get the transaction hash from it, and query the Tangle for it")
    }

    /// Returns the hash of a milestone.
    pub fn get_milestone_hash(&'static self, index: MilestoneIndex) -> Option<Hash> {
        match self.milestones.get(&index) {
            None => None,
            Some(v) => Some(*v),
        }
    }

    /// Returns whether the milestone index maps to a know milestone hash.
    pub fn contains_milestone(&'static self, index: MilestoneIndex) -> bool {
        self.milestones.contains_key(&index)
    }

    /// Retreives the solid milestone index.
    pub fn get_solid_milestone_index(&'static self) -> MilestoneIndex {
        self.solid_milestone_index.load(Ordering::Relaxed).into()
    }

    /// Updates the solid milestone index to `new_index`.
    pub fn update_solid_milestone_index(&'static self, new_index: MilestoneIndex) {
        self.solid_milestone_index.store(*new_index, Ordering::Relaxed);
    }

    /// Retreives the snapshot milestone index.
    pub fn get_snapshot_milestone_index(&'static self) -> MilestoneIndex {
        self.snapshot_milestone_index.load(Ordering::Relaxed).into()
    }

    /// Updates the snapshot milestone index to `new_index`.
    pub fn update_snapshot_milestone_index(&'static self, new_index: MilestoneIndex) {
        self.snapshot_milestone_index.store(*new_index, Ordering::Relaxed);
    }

    /// Retreives the last milestone index.
    pub fn get_last_milestone_index(&'static self) -> MilestoneIndex {
        self.last_milestone_index.load(Ordering::Relaxed).into()
    }

    /// Updates the last milestone index to `new_index`.
    pub fn update_last_milestone_index(&'static self, new_index: MilestoneIndex) {
        self.last_milestone_index.store(*new_index, Ordering::Relaxed);
    }

    /// Adds `hash` to the set of solid entry points.
    pub fn add_solid_entry_point(&'static self, hash: Hash) {
        self.solid_entry_points.insert(hash);
    }

    /// Removes `hash` from the set of solid entry points.
    pub fn remove_solid_entry_point(&'static self, hash: Hash) {
        self.solid_entry_points.remove(&hash);
    }

    /// Returns whether the transaction associated `hash` is a solid entry point.
    pub fn is_solid_entry_point(&'static self, hash: &Hash) -> bool {
        self.solid_entry_points.contains(hash)
    }

    /// Checks if the tangle is synced or not
    pub fn is_synced(&'static self) -> bool {
        self.get_solid_milestone_index() == self.get_last_milestone_index()
    }

    /// Returns the current size of the Tangle.
    pub fn size(&'static self) -> usize {
        self.vertices.len()
    }

    /// Starts a walk beginning at a `start` vertex identified by its associated transaction hash
    /// traversing its children/approvers for as long as those satisfy a given `filter`.
    ///
    /// Returns a list of descendents of `start`. It is ensured, that all elements of that list
    /// are connected through the trunk.
    pub fn trunk_walk_approvers<F>(&'static self, start: Hash, filter: F) -> Vec<(TransactionRef, Hash)>
    where
        F: Fn(&TransactionRef) -> bool,
    {
        let mut approvees = vec![];
        let mut collected = vec![];

        if let Some(approvee_ref) = self.vertices.get(&start) {
            let approvee_vtx = approvee_ref.value();
            let approvee = approvee_vtx.get_ref_to_inner();

            if filter(&approvee) {
                approvees.push(start);
                collected.push((approvee, approvee_vtx.get_id()));

                while let Some(approvee_hash) = approvees.pop() {
                    if let Some(approvers_ref) = self.approvers.get(&approvee_hash) {
                        for approver_hash in approvers_ref.value() {
                            if let Some(approver_ref) = self.vertices.get(approver_hash) {
                                let approver = approver_ref.value().get_ref_to_inner();

                                if *approver.trunk() == approvee_hash && filter(&approver) {
                                    approvees.push(*approver_hash);
                                    collected.push((approver, approver_ref.value().get_id()));
                                    // NOTE: For simplicity reasons we break here, and assume, that there can't be
                                    // a second approver that passes the filter
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        collected
    }

    /// Starts a walk beginning at a `start` vertex identified by its associated transaction hash
    /// traversing its ancestors/approvees for as long as those satisfy a given `filter`.
    ///
    /// Returns a list of ancestors of `start`. It is ensured, that all elements of that list
    /// are connected through the trunk.
    pub fn trunk_walk_approvees<F>(&'static self, start: Hash, filter: F) -> Vec<(TransactionRef, Hash)>
    where
        F: Fn(&TransactionRef) -> bool,
    {
        let mut approvers = vec![start];
        let mut collected = vec![];

        while let Some(approver_hash) = approvers.pop() {
            if let Some(approver_ref) = self.vertices.get(&approver_hash) {
                let approver_vtx = approver_ref.value();
                let approver = approver_vtx.get_ref_to_inner();

                if !filter(&approver) {
                    break;
                } else {
                    approvers.push(approver.trunk().clone());
                    collected.push((approver, approver_vtx.get_id()));
                }
            }
        }

        collected
    }

    /// Walks all approvers given a starting hash `root`.
    pub fn walk_approvees_depth_first<Mapping, Follow, Missing>(
        &'static self,
        root: Hash,
        mut map: Mapping,
        should_follow: Follow,
        mut on_missing: Missing,
    ) where
        Mapping: FnMut(&TransactionRef),
        Follow: Fn(&Vertex) -> bool,
        Missing: FnMut(&Hash),
    {
        let mut non_analyzed_hashes = Vec::new();
        let mut analyzed_hashes = HashSet::new();

        non_analyzed_hashes.push(root);

        while let Some(hash) = non_analyzed_hashes.pop() {
            if !analyzed_hashes.contains(&hash) {
                match self.vertices.get(&hash) {
                    Some(vertex) => {
                        let vertex = vertex.value();
                        let transaction = vertex.get_ref_to_inner();

                        map(&transaction);

                        if should_follow(vertex) {
                            non_analyzed_hashes.push(*transaction.branch());
                            non_analyzed_hashes.push(*transaction.trunk());
                        }
                    }
                    None => {
                        if !self.is_solid_entry_point(&hash) {
                            on_missing(&hash);
                        }
                    }
                }
                analyzed_hashes.insert(hash);
            }
        }
    }

    #[cfg(test)]
    fn num_approvers(&'static self, hash: &Hash) -> usize {
        self.approvers.get(hash).map_or(0, |r| r.value().len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    use bee_test::transaction::{create_random_attached_tx, create_random_tx};

    use async_std::sync::channel;
    use bee_bundle::{TransactionField, Value};
    use serial_test::serial;

    use async_std::task::block_on;

    #[test]
    #[serial]
    fn insert_and_contains() {
        init();
        let tangle = tangle();

        let (hash, transaction) = create_random_tx();

        assert!(block_on(tangle.insert_transaction(transaction, hash)).is_some());
        assert_eq!(1, tangle.size());
        assert!(tangle.contains_transaction(&hash));

        drop();
    }

    #[test]
    #[serial]
    fn update_and_get_snapshot_milestone_index() {
        init();
        let tangle = tangle();

        tangle.update_snapshot_milestone_index(1368160.into());

        assert_eq!(1368160, *tangle.get_snapshot_milestone_index());
        drop();
    }

    #[test]
    #[serial]
    fn update_and_get_solid_milestone_index() {
        init();
        let tangle = tangle();

        tangle.update_solid_milestone_index(1368167.into());

        assert_eq!(1368167, *tangle.get_solid_milestone_index());
        drop();
    }

    #[test]
    #[serial]
    fn update_and_get_last_milestone_index() {
        init();
        let tangle = tangle();

        tangle.update_last_milestone_index(1368168.into());

        assert_eq!(1368168, *tangle.get_last_milestone_index());
        drop();
    }

    #[test]
    #[serial]
    fn walk_trunk_approvers() {
        init();
        let (Transactions { a, d, e, .. }, Hashes { a_hash, .. }) = create_test_tangle();

        let txs = tangle().trunk_walk_approvers(a_hash, |tx| true);

        assert_eq!(3, txs.len());
        assert_eq!(a.address(), txs[0].0.address());
        assert_eq!(d.address(), txs[1].0.address());
        assert_eq!(e.address(), txs[2].0.address());

        drop();
    }

    #[test]
    #[serial]
    fn walk_trunk_approvees() {
        init();
        let (Transactions { a, d, e, .. }, Hashes { e_hash, .. }) = create_test_tangle();

        let txs = tangle().trunk_walk_approvees(e_hash, |tx| true);

        assert_eq!(3, txs.len());
        assert_eq!(e.address(), txs[0].0.address());
        assert_eq!(d.address(), txs[1].0.address());
        assert_eq!(a.address(), txs[2].0.address());

        drop();
    }

    #[test]
    #[serial]
    fn walk_approvees() {
        init();
        let (Transactions { a, d, e, .. }, Hashes { e_hash, .. }) = create_test_tangle();

        drop();
    }

    #[test]
    #[serial]
    fn walk_approvees_depth_first() {
        init();
        let (Transactions { a, b, c, d, e, .. }, Hashes { e_hash, .. }) = create_test_tangle();

        let mut addresses = vec![];

        tangle().walk_approvees_depth_first(
            e_hash,
            |tx_ref| addresses.push(tx_ref.address().clone()),
            |tx_ref| true,
            |tx_hash| (),
        );

        assert_eq!(*e.address(), addresses[0]);
        assert_eq!(*d.address(), addresses[1]);
        assert_eq!(*a.address(), addresses[2]);
        assert_eq!(*c.address(), addresses[3]);
        assert_eq!(*b.address(), addresses[4]);
        drop();
    }

    struct Transactions {
        pub a: Transaction,
        pub b: Transaction,
        pub c: Transaction,
        pub d: Transaction,
        pub e: Transaction,
    }

    struct Hashes {
        pub a_hash: Hash,
        pub b_hash: Hash,
        pub c_hash: Hash,
        pub d_hash: Hash,
        pub e_hash: Hash,
    }

    #[allow(clippy::many_single_char_names)]
    fn create_test_tangle() -> (Transactions, Hashes) {
        // a   b
        // |\ /
        // | c
        // |/|
        // d |
        //  \|
        //   e
        //
        // Trunk path from 'e':
        // e --(trunk)-> d --(trunk)-> a

        let tangle = tangle();

        let (a_hash, a) = create_random_tx();
        let (b_hash, b) = create_random_tx();
        let (c_hash, c) = create_random_attached_tx(a_hash.clone(), b_hash.clone()); // branch, trunk
        let (d_hash, d) = create_random_attached_tx(c_hash.clone(), a_hash.clone());
        let (e_hash, e) = create_random_attached_tx(c_hash.clone(), d_hash.clone());

        block_on(async {
            tangle.insert_transaction(a.clone(), a_hash).await;
            tangle.insert_transaction(b.clone(), b_hash).await;
            tangle.insert_transaction(c.clone(), c_hash).await;
            tangle.insert_transaction(d.clone(), d_hash).await;
            tangle.insert_transaction(e.clone(), e_hash).await;
        });

        assert_eq!(5, tangle.size());
        assert_eq!(2, tangle.num_approvers(&a_hash));
        assert_eq!(1, tangle.num_approvers(&b_hash));
        assert_eq!(2, tangle.num_approvers(&c_hash));
        assert_eq!(1, tangle.num_approvers(&d_hash));
        assert_eq!(0, tangle.num_approvers(&e_hash));

        (
            Transactions { a, b, c, d, e },
            Hashes {
                a_hash,
                b_hash,
                c_hash,
                d_hash,
                e_hash,
            },
        )
    }
}
