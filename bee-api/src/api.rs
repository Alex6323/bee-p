use async_trait::async_trait;

#[async_trait]
pub trait Api {

    type Params;
    type Response;

    async fn node_info() -> Self::Response;
    async fn transactions_by_bundle(params: Self::Params) -> Self::Response;
    async fn transaction_by_hash(params: Self::Params) -> Self::Response;
    async fn transactions_by_hashes(params: Self::Params) -> Self::Response;
}