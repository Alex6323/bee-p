use std::{
    any::TypeId,
    convert::Infallible,
};
use bee_common_ext::{
    worker::Worker,
    node::Node,
};
use bee_common::shutdown_stream::ShutdownStream;
use log::info;
use async_trait::async_trait;

use crate::{
    worker::storage::StorageWorker,
    tangle::MsTangle,
};

pub struct TangleWorker;

#[async_trait]
impl<N: Node> Worker<N> for TangleWorker {
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        Box::leak(Box::from(vec![TypeId::of::<TangleWorker>()]))
    }

    async fn start(
        node: &mut N,
        config: Self::Config,
    ) -> Result<Self, Self::Error> {
        info!("Starting Tangle worker...");

        let storage = node.storage().clone();
        let tangle = MsTangle::<N::Backend>::new(storage);

        node.register_resource(tangle);

        Ok(Self)
    }
}
