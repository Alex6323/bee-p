use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicPtr, Ordering},
    },
    ptr,
};
use async_std::{prelude::*, sync::{channel, Sender, Receiver}};
use dashmap::DashMap;
use bee_bundle::{
    Hash,
    Transaction,
};

pub type TransactionId = Hash;
pub type MilestoneIndex = usize;

pub struct Vertex {
    meta: VertexMeta,
    transaction: Transaction,
}

static TANGLE: AtomicPtr<Tangle> = AtomicPtr::new(ptr::null_mut());

pub fn tangle() -> &'static Tangle {
    let tangle = TANGLE.load(Ordering::Relaxed);
    if tangle.is_null() {
        panic!("Tangle cannot be null");
    } else {
        unsafe { &*tangle }
    }
}

pub fn init_tangle() {
    TANGLE.store(Box::into_raw(Tangle::new().into()), Ordering::Acquire);
}

pub struct Tangle {
    vertices: DashMap<TransactionId, Vertex>,
    unsolid_new: Sender<Hash>,
}

impl Tangle {
    pub fn new() -> Self {
        Self {
            vertices: DashMap::new(),
            unsolid_new: panic!(),
        }
    }

    pub async fn insert(&'static self, hash: Hash, v: Vertex) -> Option<VertexRef> {
        let meta = v.meta;
        if self.vertices.insert(hash, v).is_none() {
            self.unsolid_new.send(hash).await;
            Some(VertexRef {
                meta,
                tangle: self,
            })
        } else {
            None
        }
    }

    pub async fn solidify(&'static self, id: TransactionId) -> Option<()> {
        todo!()
    }

    pub async fn get_meta(&'static self, id: TransactionId) -> Option<VertexMeta> {
        todo!()
    }

    pub async fn get_meta(&'static self, id: TransactionId) -> Option<VertexMeta> {
        todo!()
    }

    /// This function is *eventually consistent* - if `true` is returned, solidification has
    /// definitely occurred. If `false` is returned, then solidification has probably not occurred,
    /// or solidification information has not yet been fully propagated.
    pub async fn is_solid(&'static self, id: TransactionId) -> Option<bool> {
        todo!()
    }

    pub async fn get_body(&'static self, id: TransactionId) -> Option<&Transaction> {
        todo!()
    }

    pub async fn get(&'static self, id: TransactionId) -> Option<VertexRef> {
        Some(VertexRef {
            meta: self.get_meta(id).await?,
            tangle: self,
        })
    }

    pub async fn contains(&'static self, id: TransactionId) -> bool {
        self.get_meta(id).await.is_some()
    }

    // Milestone stuff

    pub async fn get_milestone(&'static self, idx: MilestoneIndex) -> Option<VertexRef> {
        todo!()
    }

    pub async fn get_latest_milestone(&'static self, idx: MilestoneIndex) -> Option<VertexRef> {
        todo!()
    }
}

// Solidifier

pub struct SoldifierState {
    vert_to_approvers: HashMap<Hash, Vec<Hash>>,
    missing_to_approvers: HashMap<Hash, Vec<Arc<Hash>>>,
    unsolid_new: Receiver<Hash>,
}

impl SoldifierState {
    pub async fn worker(mut state: SoldifierState) {
        while let Some(hash) = state.unsolid_new.next().await {
            // Solidification algorithm here, write back to TANGLE
        }
    }
}

#[derive(Copy, Clone)]
pub struct VertexMeta {
    id: TransactionId,
    trunk: TransactionId,
    branch: TransactionId,
}

// VertexRef API

#[derive(Copy, Clone)]
pub struct VertexRef {
    meta: VertexMeta,
    tangle: &'static Tangle,
}

impl VertexRef {
    pub async fn get_body(&self) -> Option<&Transaction> {
        self.tangle.get_body(self.meta.id).await
    }

    pub async fn get_trunk(&self) -> Option<Self> {
        self.tangle.get(self.meta.trunk).await
    }

    pub async fn get_branch(&self) -> Option<Self> {
        self.tangle.get(self.meta.branch).await
    }
}

/*
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    rc::Rc,
};

/// A transaction hash. To be replaced later with whatever implementation is required.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct TxHash(u64);

impl TxHash {
    pub fn is_genesis(&self) -> bool {
        unimplemented!()
    }
}

/// A transaction. Cannot be mutated once created.
pub struct Tx {
    trunk: TxHash,
    branch: TxHash,
    body: (),
}

impl Tx {
    pub fn trunk_hash(&self) -> TxHash {
        self.trunk
    }

    pub fn branch_hash(&self) -> TxHash {
        self.branch
    }

    pub fn approvee_hashes(&self) -> [TxHash; 2] {
        [self.trunk, self.branch]
    }
}

/// A vertex within the Tangle. Each vertex represents a transaction and its associated metadata.
pub struct Vertex {
    hash: TxHash,
    tx: Tx,
    solid: bool,
}

impl Vertex {
    pub fn new(hash: TxHash, tx: Tx) -> Self {
        Self {
            hash,
            tx,
            solid: false,
        }
    }

    /// This method is private because all solidification should occur via the solidification
    /// algorithm automatically.
    fn set_solid(&mut self) {
        self.solid = true;
    }

    pub fn is_solid(&self) -> bool {
        self.solid
    }

    pub fn tx(&self) -> &Tx {
        &self.tx
    }
}

impl Deref for Vertex {
    type Target = Tx;

    fn deref(&self) -> &Tx {
        &self.tx
    }
}

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

    /// Attempt to perform solidification upon a node (and its approvers). This method is private
    /// because it is automatically run whenever the tangle is updated with new information
    fn try_solidify(&mut self, root: TxHash) {
        // Prevent borrow errors by borrowing the fields independently
        let vertices = &mut self.vertices;
        let txs_to_approvers = &self.txs_to_approvers;

        // The algorithm is recursive, but we don't want to use the stack
        let mut stack = vec![root];
        while let Some(current_vert) = stack.pop() {
            if let Some(approvee_hashes) = vertices
                .get(&current_vert)
                .filter(|v| !v.is_solid())
                .map(|v| v.approvee_hashes())
            {
                if approvee_hashes
                    // For each of the current root's approvees...
                    .iter()
                    // ...ensure that they are all solid...
                    .all(|a| {
                        vertices.get(&a).map(|a| a.is_solid()).unwrap_or(false) || a.is_genesis()
                    })
                {
                    // We can now solidify the current root since we know all approvees are solid
                    vertices
                        .get_mut(&current_vert)
                        .unwrap() // Can't fail
                        .set_solid();

                    // Now, propagate this information to the approvers of the current root by
                    // running the algorithm again for each of them
                    for approver in txs_to_approvers
                        .get(&current_vert)
                        .iter()
                        .map(|approvers| approvers.iter())
                        .flatten()
                    {
                        // Push the approver to the stack as the next vertex to consider
                        stack.push(*approver);
                    }
                }
            }
        }
    }
}

pub struct VertexRef<'a> {
    tangle: &'a Tangle,
    hash: TxHash,
}

impl<'a> VertexRef<'a> {
    pub fn exists(&self) -> bool {
        self.tangle.contains(self.hash)
    }

    pub fn trunk(&'a self) -> Option<Self> {
        let trunk_hash = self.tx.trunk;
        self.tangle.get(trunk_hash)
    }

    pub fn branch(&'a self) -> Option<Self> {
        let branch_hash = self.tx.branch;
        self.tangle.get(branch_hash)
    }
}

impl<'a> Deref for VertexRef<'a> {
    type Target = Vertex;

    fn deref(&self) -> &Self::Target {
        self.tangle.vertices.get(&self.hash).unwrap()
    }
}

pub struct VertexRefMut<'a> {
    tangle: &'a mut Tangle,
    hash: TxHash,
}

impl<'a> VertexRefMut<'a> {
    pub fn trunk(&'a mut self) -> Option<Self> {
        let trunk_hash = self.tx.trunk;
        self.tangle.get_mut(trunk_hash)
    }

    pub fn branch(&'a mut self) -> Option<Self> {
        let branch_hash = self.tx.branch;
        self.tangle.get_mut(branch_hash)
    }
}

impl<'a> Deref for VertexRefMut<'a> {
    type Target = Vertex;

    fn deref(&self) -> &Self::Target {
        self.tangle.vertices.get(&self.hash).unwrap()
    }
}

impl<'a> DerefMut for VertexRefMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.tangle.vertices.get_mut(&self.hash).unwrap()
    }
}

impl<'a> VertexRefMut<'a> {
    pub fn do_for(&self, f: impl FnOnce(&Vertex)) {
        f(self.tangle.vertices.get(&self.hash).unwrap());
    }

    pub fn do_for_mut(&mut self, f: impl FnOnce(&mut Vertex)) {
        f(self.tangle.vertices.get_mut(&self.hash).unwrap());
    }
}

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
