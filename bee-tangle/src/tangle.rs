//! Module that provides the [`Tangle`] struct.

use crate::{
    vertex::{
        Vertex,
        VertexMeta,
        VertexRef,
    },
    MilestoneIndex,
    TransactionId,
};

use async_std::sync::Sender;
use dashmap::DashMap;

use bee_bundle::{
    Hash,
    Transaction,
};

type DashSet<T> = DashMap<T, ()>;

/// A datastructure based on a directed acyclic graph (DAG).
pub struct Tangle {
    vertices: DashMap<TransactionId, Vertex>,
    unsolid_new: Sender<Hash>,
    solid_entry_points: DashSet<Hash>,
}

impl Tangle {
    /// Constructor.
    pub(crate) fn new(unsolid_new: Sender<Hash>) -> Self {
        Self {
            vertices: DashMap::new(),
            unsolid_new,
            solid_entry_points: DashSet::new(),
        }
    }

    /// Inserts a transaction.
    pub async fn insert_transaction(&'static self, transaction: Transaction, hash: Hash) -> Option<VertexRef> {
        let vertex = Vertex::from(transaction, hash);

        self.insert(hash, vertex).await
    }

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

    async fn solidify(&'static self, _id: TransactionId) -> Option<()> {
        todo!()
    }

    /// Returns meta data about transaction, if it's available in the local Tangle.
    pub async fn get_meta(&'static self, _id: TransactionId) -> Option<VertexMeta> {
        todo!()
    }

    /// Returns a reference to a transaction, if it's available in the local Tangle.
    pub async fn get_body(&'static self, _id: TransactionId) -> Option<&Transaction> {
        todo!()
    }

    /// This function is *eventually consistent* - if `true` is returned, solidification has
    /// definitely occurred. If `false` is returned, then solidification has probably not occurred,
    /// or solidification information has not yet been fully propagated.
    pub async fn is_solid(&'static self, _id: TransactionId) -> Option<bool> {
        todo!()
    }

    /// Returns a [`VertexRef`] linked to a transaction, if it's available in the local Tangle.
    pub async fn get(&'static self, id: TransactionId) -> Option<VertexRef> {
        Some(VertexRef {
            meta: self.get_meta(id).await?,
            tangle: self,
        })
    }

    /// Returns whether the transaction is stored in the local Tangle.
    pub async fn contains(&'static self, id: TransactionId) -> bool {
        self.get_meta(id).await.is_some()
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
    pub fn add_solid_entry_point(&self, hash: Hash) {
        self.solid_entry_points.insert(hash, ());
    }

    /// Removes `hash` from the set of solid entry points.
    pub fn rmv_solid_entry_point(&self, hash: Hash) {
        self.solid_entry_points.remove(&hash);
    }

    /// Returns whether the transaction associated `hash` is a solid entry point.
    pub fn is_solid_entry_point(&self, hash: &Hash) -> bool {
        self.solid_entry_points.contains_key(hash)
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

    use async_std::sync::channel;

    #[test]
    fn new_tangle() {
        let (sender, _receiver) = channel::<Hash>(1000);

        let _ = Tangle::new(sender);
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
