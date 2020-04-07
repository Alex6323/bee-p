//! Module that provides the [`Tangle`] struct.

use crate::{
    milestone::MilestoneIndex,
    vertex::{
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
    unsolid_new: Sender<Hash>,
    solid_entry_points: DashSet<Hash>,
    first_solid_milestone: AtomicU32,
    last_solid_milestone: AtomicU32,
    last_milestone: AtomicU32,
}

impl Tangle {
    /// Creates a new `Tangle`.
    pub(crate) fn new(unsolid_new: Sender<Hash>) -> Self {
        Self {
            vertices: DashMap::new(),
            unsolid_new,
            solid_entry_points: DashSet::new(),
            first_solid_milestone: AtomicU32::new(0),
            last_solid_milestone: AtomicU32::new(0),
            last_milestone: AtomicU32::new(0),
        }
    }

    /// Inserts a transaction.
    ///
    /// TODO: there is no guarantee `hash` belongs to `transaction`. User responsibility?
    pub async fn insert_transaction(&'static self, transaction: Transaction, hash: Hash) -> Option<VertexRef> {
        let vertex = Vertex::from(transaction, hash);

        self.insert(hash, vertex).await
    }

    #[inline(always)]
    async fn insert(&'static self, hash: Hash, vertex: Vertex) -> Option<VertexRef> {
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

    async fn get_meta(&'static self, hash: &Hash) -> Option<VertexMeta> {
        self.vertices.get(hash).map(|v| v.meta)
    }

    /// Returns a reference to a transaction, if it's available in the local Tangle.
    pub async fn get_transaction(&'static self, _hash: &Hash) -> Option<&Transaction> {
        todo!()
    }

    /// This function is *eventually consistent* - if `true` is returned, solidification has
    /// definitely occurred. If `false` is returned, then solidification has probably not occurred,
    /// or solidification information has not yet been fully propagated.
    pub async fn is_solid(&'static self, _hash: Hash) -> Option<bool> {
        todo!()
    }

    /// Returns a [`VertexRef`] linked to a transaction, if it's available in the local Tangle.
    pub async fn get(&'static self, hash: &Hash) -> Option<VertexRef> {
        Some(VertexRef {
            meta: self.get_meta(&hash).await?,
            tangle: self,
        })
    }

    ///  Returns a [`VertexRef`] linked to the specified milestone, if it's available in the local Tangle.
    pub async fn get_milestone(&'static self, _idx: MilestoneIndex) -> Option<VertexRef> {
        todo!()
    }

    /// Returns a [`VertexRef`] linked to the specified milestone, if it's available in the local Tangle.
    pub async fn get_latest_milestone(&'static self, _idx: MilestoneIndex) -> Option<VertexRef> {
        todo!()
    }

    /// Adds `hash` to the set of solid entry points.
    pub fn add_solid_entry_point(&'static self, hash: Hash) {
        self.solid_entry_points.insert(hash);
    }

    /// Removes `hash` from the set of solid entry points.
    pub fn rmv_solid_entry_point(&'static self, hash: Hash) {
        self.solid_entry_points.remove(&hash);
    }

    /// Returns whether the transaction associated `hash` is a solid entry point.
    pub fn is_solid_entry_point(&'static self, hash: &Hash) -> bool {
        self.solid_entry_points.contains(hash)
    }

    /// Updates the first solid milestone index to `new_index`.
    pub fn update_first_solid_milestone_index(&'static self, new_index: MilestoneIndex) {
        self.first_solid_milestone.store(*new_index, Ordering::Relaxed);
    }

    /// Updates the last solid milestone index to `new_index`.
    pub fn update_last_solid_milestone_index(&'static self, new_index: MilestoneIndex) {
        self.last_solid_milestone.store(*new_index, Ordering::Relaxed);
    }

    /// Updates the last milestone index to `new_index`.
    pub fn update_last_milestone_index(&'static self, new_index: MilestoneIndex) {
        self.last_milestone.store(*new_index, Ordering::Relaxed);
    }

    /// Retreives the first solid milestone index.
    pub fn get_first_solid_milestone_index(&'static self) -> MilestoneIndex {
        self.first_solid_milestone.load(Ordering::Relaxed).into()
    }

    /// Retreives the last solid milestone index.
    pub fn get_last_solid_milestone_index(&'static self) -> MilestoneIndex {
        self.last_solid_milestone.load(Ordering::Relaxed).into()
    }

    /// Retreives the last milestone index.
    pub fn get_last_milestone_index(&'static self) -> MilestoneIndex {
        self.last_milestone.load(Ordering::Relaxed).into()
    }

    /// Returns the current size of the Tangle.
    pub fn size(&'static self) -> usize {
        self.vertices.len()
    }
}

/*
/// The main Tangle structure. Usually, this type is used as a singleton.
#[derive(Default)]
pub struct Tangle {
    vertices: HashMap<TxHash, Vertex>,
    txs_to_approvers: HashMap<TxHash, Vec<TxHash>>,
    missing_to_approvers: HashMap<TxHash, Vec<Rc<TxHash>>>,
}

impl Tangle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contains(&self, hash: TxHash) -> bool {
        self.vertices.contains_key(&hash)
    }

    /// Get an immutable handle to the transaction with the given hash.
    pub fn get(&self, hash: TxHash) -> Option<VertexRef> {
        if self.contains(hash) {
            Some(VertexRef {
                hash: hash,
                tangle: self,
            })
        } else {
            None
        }
    }

    /// Get a mutable handle to the transaction with the given hash.
    pub fn get_mut(&mut self, hash: TxHash) -> Option<VertexRefMut> {
        if self.contains(hash) {
            Some(VertexRefMut {
                hash: hash,
                tangle: self,
            })
        } else {
            None
        }
    }

    /// Insert a vertex into the Tangle, automatically triggering the solidification algorithm.
    pub fn insert(&mut self, vert: Vertex) -> VertexRefMut {
        let new_hash = vert.hash;
        let new_approvees = vert.approvee_hashes();

        // Don't re-insert a vertex
        if !self.contains(new_hash) {
            // Perform the tangle insertion
            self.vertices.insert(new_hash, vert);
            new_approvees
                .iter()
                .for_each(|a| self.txs_to_approvers.entry(*a).or_default().push(new_hash));

            // Does the new vertex approve vertices that we don't yet know about?
            if new_approvees
                // Do any of the new vertex's approvees...
                .iter()
                // ...not exist yet?
                .any(|approvee| !self.contains(*approvee))
            {
                let new_rc = Rc::new(new_hash);
                // For each approvee of the inserted vertex...
                let vertices = &self.vertices;
                let missing_to_approvers = &mut self.missing_to_approvers;
                new_approvees
                    .iter()
                    // ...check to see whether it's missing from the tangle...
                    .filter(|approvee| !vertices.contains_key(*approvee))
                    // ...and remember that visiting it is work we need to do later...
                    .for_each(|approvee| {
                        missing_to_approvers
                            .entry(*approvee)
                            .or_default()
                            // ...by associating it with the missing approvee.
                            .push(new_rc.clone())
                    });
            }

            // Attempt to propagate solidification information based on the new
            // information the inserted vertex has provided us with. We do this
            // by checking to see whether any approvers were waiting upon this vertex.
            self.missing_to_approvers
                .remove(&new_hash)
                .into_iter()
                .flatten()
                .filter_map(|hash| Rc::try_unwrap(hash).ok())
                .for_each(|hash| self.try_solidify(hash));
        }

        self.get_mut(new_hash).unwrap() // Can't fail, we just inserted it
    }

}
*/

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

/*
#[cfg(test)]
mod tests {
    use super::*;

    /*
    #[test]
    fn mutate() {
        let mut tangle = Tangle::default();

        let hash = unimplemented!();

        let vertex = tangle.get_mut(hash);

        vertex.set_solid();

        vertex.do_for(|vertex| {
            println!("Solid: {:?}", vertex.is_solid());
            println!("Trunk: {:?}", vertex.trunk_hash());
            println!("Branch: {:?}", vertex.branch_hash());
        });
    }
    */
}
*/
