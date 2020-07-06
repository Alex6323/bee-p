use async_trait::async_trait;

#[async_trait]
pub trait Api {
    type NodeInfoApiResponse;
    type TransactionsByBundleApiParams;
    type TransactionsByBundleApiResponse;
    type TransactionByHashApiParams;
    type TransactionByHashApiResponse;
    type TransactionsByHashesApiParams;
    type TransactionsByHashesApiResponse;

    async fn node_info() -> Self::NodeInfoApiResponse;
    async fn transactions_by_bundle(params: Self::TransactionsByBundleApiParams) -> Self::TransactionsByBundleApiResponse;
    async fn transaction_by_hash(params: Self::TransactionByHashApiParams) -> Self::TransactionByHashApiResponse;
    async fn transactions_by_hashes(params: Self::TransactionsByHashesApiParams) -> Self::TransactionsByHashesApiResponse;
}
