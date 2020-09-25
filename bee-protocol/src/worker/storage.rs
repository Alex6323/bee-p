use bee_common_ext::{
    worker::Worker,
    node::Node,
};
use bee_common::shutdown_stream::ShutdownStream;
use async_trait::async_trait;
use std::error::Error;

pub struct StorageWorker;

#[async_trait]
impl<N: Node> Worker<N> for StorageWorker {
    type Config = String; // TODO: Replace with N::Backend::Config
    type Error = Box<dyn Error>;
    type Event = ();
    type Receiver = ShutdownStream<Fuse<Interval>>;

    async fn start(
        self,
        receiver: Self::Receiver,
        node: Arc<N>,
    ) -> Result<(), Self::Error> {
        let config_path = todo!();

        let backend = N::Backend::start(config_path).await?;

        node.register_resource(backend);
    }
}
