use crate::{
    vertex::{
        Vertex,
        VertexMeta,
        VertexRef,
    },
    Hash,
    MilestoneIndex,
    TransactionId,
};

use async_std::sync::Sender;
use dashmap::DashMap;

use bee_bundle::Transaction;

pub struct Tangle {
    vertices: DashMap<TransactionId, Vertex>,
    unsolid_new: Sender<Hash>,
}

impl Tangle {
    pub fn new(unsolid_new: Sender<Hash>) -> Self {
        Self {
            vertices: DashMap::new(),
            unsolid_new,
        }
    }

    pub async fn insert(&'static self, hash: Hash, v: Vertex) -> Option<VertexRef> {
        let meta = v.meta;

        if self.vertices.insert(hash, v).is_none() {
            self.unsolid_new.send(hash).await;
            Some(VertexRef { meta, tangle: self })
        } else {
            None
        }
    }

    pub async fn solidify(&'static self, _id: TransactionId) -> Option<()> {
        todo!()
    }

    pub async fn get_meta(&'static self, _id: TransactionId) -> Option<VertexMeta> {
        todo!()
    }

    /// This function is *eventually consistent* - if `true` is returned, solidification has
    /// definitely occurred. If `false` is returned, then solidification has probably not occurred,
    /// or solidification information has not yet been fully propagated.
    pub async fn is_solid(&'static self, _id: TransactionId) -> Option<bool> {
        todo!()
    }

    pub async fn get_body(&'static self, _id: TransactionId) -> Option<&Transaction> {
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

    pub async fn get_milestone(&'static self, _idx: MilestoneIndex) -> Option<VertexRef> {
        todo!()
    }

    pub async fn get_latest_milestone(&'static self, _idx: MilestoneIndex) -> Option<VertexRef> {
        todo!()
    }
}

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
