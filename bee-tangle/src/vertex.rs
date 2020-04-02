use crate::{
    tangle::Tangle,
    TransactionId,
};

use bee_bundle::Transaction;

pub struct Vertex {
    pub(crate) meta: VertexMeta,
    _transaction: Transaction,
}

#[derive(Copy, Clone)]
pub struct VertexMeta {
    id: TransactionId,
    trunk: TransactionId,
    branch: TransactionId,
}

#[derive(Copy, Clone)]
pub struct VertexRef {
    pub(crate) meta: VertexMeta,
    pub(crate) tangle: &'static Tangle,
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
