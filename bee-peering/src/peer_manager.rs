use async_trait::async_trait;

#[async_trait]
pub trait PeerManager {
    async fn run(self);
}
