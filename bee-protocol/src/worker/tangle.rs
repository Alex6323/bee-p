use std::any::TypeId;
use bee_common_ext::{
    worker::Worker,
    node::Node,
};
use bee_common::shutdown_stream::ShutdownStream;
use async_trait::async_trait;

use crate::{
    worker::storage::StorageWorker,
    tangle::MsTangle,
};

pub struct TangleWorker;

#[async_trait]
impl<N: Node> Worker<N> for TangleWorker {
    const DEPS: &'static [TypeId] = &[TypeId::of::<TangleWorker>()];

    type Config = ();
    type Error = WorkerError;
    type Event = ();
    type Receiver = ShutdownStream<Fuse<Interval>>;

    async fn start(
        self,
        receiver: Self::Receiver,
        node: Arc<N>,
    ) -> Result<(), Self::Error> {
        let tangle = MsTangle::<N::Backend>::new();

        node.register_resource(tangle);
    }
}
