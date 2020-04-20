//! Module that provides the [`Tangle`] struct.

use crate::{
    milestone::MilestoneIndex,
    vertex::{
        TransactionRef,
        Vertex,
        VertexMeta,
        VertexRef,
    },
};

use async_std::sync::{
    Arc,
    Sender,
};
use dashmap::{
    mapref::entry::Entry,
    DashMap,
    DashSet,
};

use bee_bundle::{
    Hash,
    Transaction,
};

use std::sync::atomic::{
    AtomicU32,
    Ordering,
};

/// A datastructure based on a directed acyclic graph (DAG).
pub struct Tangle {
    vertices: DashMap<Hash, Vertex>,
    approvers: DashMap<Hash, Vec<Hash>>,
    unsolid_new: Sender<Hash>,
    solid_entry_points: DashSet<Hash>,
    milestones: DashMap<MilestoneIndex, Hash>,
    first_solid_milestone: AtomicU32,
    last_solid_milestone: AtomicU32,
    last_milestone: AtomicU32,
}

impl Tangle {
    /// Creates a new `Tangle`.
    pub(crate) fn new(unsolid_new: Sender<Hash>) -> Self {
        Self {
            vertices: DashMap::new(),
            approvers: DashMap::new(),
            unsolid_new,
            solid_entry_points: DashSet::new(),
            milestones: DashMap::new(),
            first_solid_milestone: AtomicU32::new(0),
            last_solid_milestone: AtomicU32::new(0),
            last_milestone: AtomicU32::new(0),
        }
    }

    /// Inserts a transaction.
    ///
    /// TODO: there is no guarantee `hash` belongs to `transaction`. User responsibility?
    pub async fn insert_transaction(&'static self, transaction: Transaction, hash: Hash) -> Option<VertexRef> {
        match self.approvers.entry(*transaction.trunk()) {
            Entry::Occupied(mut entry) => {
                let values = entry.get_mut();
                values.push(hash.clone());
            }
            Entry::Vacant(entry) => {
                entry.insert(vec![hash.clone()]);
                ()
            }
        }

        match self.approvers.entry(*transaction.branch()) {
            Entry::Occupied(mut entry) => {
                let values = entry.get_mut();
                values.push(hash.clone());
            }
            Entry::Vacant(entry) => {
                entry.insert(vec![hash.clone()]);
                ()
            }
        }

        let vertex = Vertex::from(transaction, hash);
        let meta = vertex.meta;

        // TODO: not sure if we want replacement of vertices
        if self.vertices.insert(hash, vertex).is_none() {
            self.unsolid_new.send(hash).await;

            Some(VertexRef { meta, tangle: self })
        } else {
            None
        }
    }

    /// Returns whether the transaction is stored in the Tangle.
    pub fn contains_transaction(&'static self, hash: &Hash) -> bool {
        self.vertices.contains_key(hash)
    }

    async fn solidify(&'static self, _hash: Hash) -> Option<()> {
        todo!()
    }

    fn get_meta(&'static self, hash: &Hash) -> Option<VertexMeta> {
        self.vertices.get(hash).map(|v| v.meta)
    }

    /// Returns a reference to a transaction, if it's available in the local Tangle.
    pub fn get_transaction(&'static self, hash: &Hash) -> Option<TransactionRef> {
        self.vertices.get(hash).map(|v| v.get_transaction())
    }

    /// This function is *eventually consistent* - if `true` is returned, solidification has
    /// definitely occurred. If `false` is returned, then solidification has probably not occurred,
    /// or solidification information has not yet been fully propagated.
    pub async fn is_solid(&'static self, _hash: Hash) -> Option<bool> {
        todo!()
    }

    /// Returns a [`VertexRef`] linked to a transaction, if it's available in the local Tangle.
    pub fn get(&'static self, hash: &Hash) -> Option<VertexRef> {
        Some(VertexRef {
            meta: self.get_meta(&hash)?,
            tangle: self,
        })
    }

    ///  Returns a [`VertexRef`] linked to the specified milestone, if it's available in the local Tangle.
    pub fn get_milestone(&'static self, index: &MilestoneIndex) -> Option<VertexRef> {
        match self.get_milestone_hash(index) {
            None => None,
            Some(hash) => Some(VertexRef {
                meta: self.get_meta(&hash)?,
                tangle: self,
            }),
        }
    }

    /// Returns a [`VertexRef`] linked to the specified milestone, if it's available in the local Tangle.
    pub fn get_latest_milestone(&'static self, _idx: MilestoneIndex) -> Option<VertexRef> {
        todo!()
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

    /// Adds the `hash` of a milestone identified by its milestone `index`.
    pub fn add_milestone_hash(&'static self, index: MilestoneIndex, hash: Hash) {
        self.milestones.insert(index, hash);
    }

    /// Removes the hash of a milestone.
    pub fn remove_milestone_hash(&'static self, index: &MilestoneIndex, hash: Hash) {
        self.milestones.remove(index);
    }

    /// Returns the hash of a milestone.
    pub fn get_milestone_hash(&'static self, index: &MilestoneIndex) -> Option<Hash> {
        match self.milestones.get(index) {
            None => None,
            Some(v) => Some(*v),
        }
    }

    /// Returns whether the milestone index maps to a know milestone hash.
    pub fn contains_milestone(&'static self, index: &MilestoneIndex) -> bool {
        self.milestones.contains_key(index)
    }

    /// Retreives the first solid milestone index.
    pub fn get_first_solid_milestone_index(&'static self) -> MilestoneIndex {
        self.first_solid_milestone.load(Ordering::Relaxed).into()
    }

    /// Updates the first solid milestone index to `new_index`.
    pub fn update_first_solid_milestone_index(&'static self, new_index: MilestoneIndex) {
        self.first_solid_milestone.store(*new_index, Ordering::Relaxed);
    }

    /// Retreives the last solid milestone index.
    pub fn get_last_solid_milestone_index(&'static self) -> MilestoneIndex {
        self.last_solid_milestone.load(Ordering::Relaxed).into()
    }

    /// Updates the last solid milestone index to `new_index`.
    pub fn update_last_solid_milestone_index(&'static self, new_index: MilestoneIndex) {
        self.last_solid_milestone.store(*new_index, Ordering::Relaxed);
    }

    /// Retreives the last milestone index.
    pub fn get_last_milestone_index(&'static self) -> MilestoneIndex {
        self.last_milestone.load(Ordering::Relaxed).into()
    }

    /// Updates the last milestone index to `new_index`.
    pub fn update_last_milestone_index(&'static self, new_index: MilestoneIndex) {
        self.last_milestone.store(*new_index, Ordering::Relaxed);
    }

    /// Checks if the tangle is synced or not
    pub fn is_synced(&'static self) -> bool {
        self.get_last_solid_milestone_index() == self.get_last_milestone_index()
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
    pub fn trunk_walk_approvers<F>(&'static self, start: Hash, filter: F) -> Vec<TransactionRef>
    where
        F: Fn(&TransactionRef) -> bool,
    {
        let mut approvees = vec![];
        let mut collected = vec![];

        if let Some(approvee_ref) = self.vertices.get(&start) {
            let approvee = approvee_ref.value().get_transaction();

            if filter(&approvee) {
                approvees.push(start);
                collected.push(approvee);
            }

            while let Some(approvee_hash) = approvees.pop() {
                if let Some(approvers_ref) = self.approvers.get(&approvee_hash) {
                    for approver_hash in approvers_ref.value() {
                        if let Some(approver_ref) = self.vertices.get(approver_hash) {
                            let approver = approver_ref.value().get_transaction();

                            if *approver.trunk() == approvee_hash && filter(&approver) {
                                approvees.push(*approver_hash);
                                collected.push(approver);
                                // NOTE: For simplicity reasons we break here, and assume, that there can't be
                                // a second approver that passes the filter
                                break;
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
    pub fn trunk_walk_approvees<F>(&'static self, start: Hash, filter: F) -> Vec<TransactionRef>
    where
        F: Fn(&TransactionRef) -> bool,
    {
        let mut approvers = vec![start];
        let mut collected = vec![];

        while let Some(approver_hash) = approvers.pop() {
            if let Some(approver_ref) = self.vertices.get(&approver_hash) {
                let approver = approver_ref.value().get_transaction();

                if !filter(&approver) {
                    break;
                } else {
                    approvers.push(approver.trunk().clone());
                    collected.push(approver);
                }
            }
        }

        collected
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    use bee_test::transaction::create_random_tx;

    use async_std::sync::channel;
    use bee_bundle::{
        TransactionField,
        Value,
    };
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
    fn update_and_get_first_solid_milestone_index() {
        init();
        let tangle = tangle();

        tangle.update_first_solid_milestone_index(1368160.into());

        assert_eq!(1368160, *tangle.get_first_solid_milestone_index());
        drop();
    }

    #[test]
    #[serial]
    fn update_and_get_last_solid_milestone_index() {
        init();
        let tangle = tangle();

        tangle.update_last_solid_milestone_index(1368167.into());

        assert_eq!(1368167, *tangle.get_last_solid_milestone_index());
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
}
