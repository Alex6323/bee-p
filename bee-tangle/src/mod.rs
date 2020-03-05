use bee_bundle::*;
use bee_storage_sqlx::sqlx::SqlxBackendStorage as StorageBackendImpl;
use bee_storage_sqlx::sqlx::errors;
use bee_storage::StorageBackend;

use std::{
    error::Error as StdError,
    fmt,
};


#[derive(Debug, Clone)]
pub enum TangleError {
    SqlxError(String),
    UnknownError,
    //...
}


impl fmt::Display for TangleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TangleError::SqlxError(ref reason) => write!(f, "Sqlx core error: {:?}", reason),
            TangleError::UnknownError => write!(f, "Unknown error"),
        }
    }
}

// Allow this type to be treated like an error
impl StdError for TangleError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            _ => None,
        }
    }
}

impl From<errors::SqlxBackendError> for TangleError {
    #[inline]
    fn from(err: errors::SqlxBackendError) -> Self {
        TangleError::SqlxError(err.to_string())
    }
}

use std::{
    collections::HashMap,
    collections::HashSet,
    ops::{Deref, DerefMut},
    rc::Rc,
};


use futures::executor::block_on;


/// A vertex within the Tangle. Each vertex represents a transaction and its associated metadata.
pub struct Vertex {
    hash: bee_bundle::Hash,
    tx: bee_bundle::Transaction,
    solid: bool,
}

impl Vertex {
    pub fn new(hash: bee_bundle::Hash, tx: bee_bundle::Transaction) -> Self {
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

    pub fn tx(&self) -> &bee_bundle::Transaction {
        &self.tx
    }
}

impl Deref for Vertex {
    type Target = bee_bundle::Transaction;

    fn deref(&self) -> &bee_bundle::Transaction {
        &self.tx
    }
}

/// The main Tangle structure. Usually, this type is used as a singleton.
#[derive(Default)]
pub struct Tangle {

    //TODO - might want to change to Box<dyn StorageBackend<StorageError=SomeError>>
    storage: Option<StorageBackendImpl>,
    //TODO - either remove (and use storage instead) or replace with cache
    vertices: HashMap<bee_bundle::Hash, Vertex>,
    txs_to_approvers: HashMap<bee_bundle::Hash, HashSet<bee_bundle::Hash>>,
    missing_to_approvers: HashMap<bee_bundle::Hash, HashSet<Rc<bee_bundle::Hash>>>,
}


impl Drop for Tangle {
    fn drop(&mut self) {
        block_on(self.storage.as_mut().unwrap().destroy_connection());
    }
}


impl Tangle {
    pub fn new() -> Self {
        let mut tangle = Self::default();
        tangle.storage = Some(StorageBackendImpl::new());
        tangle

    }

    pub async fn load(&mut self, db_url : &str) -> Result<(),TangleError> {

        let storage : &mut StorageBackendImpl = self.storage.as_mut().unwrap();
        storage.establish_connection(db_url).await?;


        self.txs_to_approvers =
        storage.map_existing_transaction_hashes_to_approvers()?;
        let mut all_hashes = HashSet::new();
        for key in self.txs_to_approvers.keys() {
            all_hashes.insert(key.clone());
        }
        self.missing_to_approvers = storage.map_missing_transaction_hashes_to_approvers(all_hashes)?;
        Ok(())
    }

    pub fn contains(&self, hash: &bee_bundle::Hash) -> bool {
        let mut found = false;
        if self.vertices.contains_key(hash) {
            found = true;
        }

        if (!found){
            //let storage : &mut StorageBackendImpl = self.storage.as_mut().unwrap();
           /* found = match storage.find_transaction(hash) {

                Err(reason) => false,
                OK => true
            }*/
        }


        found
    }

    /// Get an immutable handle to the transaction with the given hash.
    pub fn get(&self, hash: &bee_bundle::Hash) -> Option<VertexRef> {
        if self.contains(hash) {
            Some(VertexRef {
                hash: hash.clone(),
                tangle: self,
            })
        } else {
            None
        }
    }

    /// Get a mutable handle to the transaction with the given hash.
    pub fn get_mut(&mut self, hash: &bee_bundle::Hash) -> Option<VertexRefMut> {
        if self.contains(hash) {
            Some(VertexRefMut {
                hash: hash.clone(),
                tangle: self,
            })
        } else {
            None
        }
    }

    /// Insert a vertex into the Tangle, automatically triggering the solidification algorithm.
    pub fn insert(&mut self, vert: Vertex) -> VertexRefMut {
        let new_hash = vert.hash.clone();
        let new_approvees = [vert.branch().clone(), vert.trunk().clone()];

        // Don't re-insert a vertex
        if !self.contains(&new_hash) {
            // Perform the tangle insertion
            self.vertices.insert(new_hash.clone(), vert);
            new_approvees
                .iter()
                .for_each(|a| {
                    self.txs_to_approvers.entry(a.to_owned()).or_default().insert(new_hash.clone());
                });

            // Does the new vertex approve vertices that we don't yet know about?
            if new_approvees
                // Do any of the new vertex's approvees...
                .iter()
                // ...not exist yet?
                .any(|approvee| !self.contains(&*approvee))
            {
                let new_rc = Rc::new(new_hash.clone());
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
                            .entry(approvee.to_owned())
                            .or_default()
                            // ...by associating it with the missing approvee.
                            .insert(new_rc.clone());
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

        self.get_mut(&new_hash).unwrap() // Can't fail, we just inserted it
    }

    /// Attempt to perform solidification upon a node (and its approvers). This method is private
    /// because it is automatically run whenever the tangle is updated with new information
    fn try_solidify(&mut self, root: bee_bundle::Hash) {
        // Prevent borrow errors by borrowing the fields independently
        let vertices = &mut self.vertices;
        let txs_to_approvers = &self.txs_to_approvers;

        // The algorithm is recursive, but we don't want to use the stack
        let mut stack = vec![root];
        while let Some(current_vert) = stack.pop() {
            if let Some(approvee_hashes) = vertices
                .get(&current_vert)
                .filter(|v| !v.is_solid())
                .map(|v| [v.branch().to_owned(), v.trunk().to_owned()])
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
                            stack.push(approver.clone());
                        }
                }
            }
        }
    }
}

pub struct VertexRef<'a> {
    tangle: &'a Tangle,
    hash: bee_bundle::Hash,
}

impl<'a> VertexRef<'a> {
    pub fn exists(&self) -> bool {
        self.tangle.contains(&self.hash)
    }

    pub fn trunk(&'a self) -> Option<Self> {
        let trunk_hash = self.tx.trunk();
        self.tangle.get(trunk_hash)
    }

    pub fn branch(&'a self) -> Option<Self> {
        let branch_hash = self.tx.branch();
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
    hash: bee_bundle::Hash,
}

impl<'a> VertexRefMut<'a> {
    pub fn trunk(&'a mut self) -> Option<Self> {
        let trunk_hash = self.tx.trunk().clone();
        self.tangle.get_mut(&trunk_hash)
    }

    pub fn branch(&'a mut self) -> Option<Self> {
        let branch_hash = self.tx.branch().clone();
        self.tangle.get_mut(&branch_hash)
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

