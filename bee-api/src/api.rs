use async_trait::async_trait;

#[async_trait]
pub trait Api {
    type NodeInfoResponse;
    type TransactionsByBundleParams;
    type TransactionsByBundleResponse;
    type TransactionByHashParams;
    type TransactionByHashResponse;
    type TransactionsByHashesParams;
    type TransactionsByHashesResponse;

    async fn node_info() -> Self::NodeInfoResponse;
    async fn transactions_by_bundle(params: Self::TransactionsByBundleParams) -> Self::TransactionsByBundleResponse;
    async fn transaction_by_hash(params: Self::TransactionByHashParams) -> Self::TransactionByHashResponse;
    async fn transactions_by_hashes(params: Self::TransactionsByHashesParams) -> Self::TransactionsByHashesResponse;
}
