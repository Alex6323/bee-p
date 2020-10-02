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
        Box::leak(Box::from(vec![TypeId::of::<StorageWorker>()]))
    }

    async fn start(
        node: &mut N,
        config: Self::Config,
    ) -> Result<Self, Self::Error> {

        let storage = node.storage().clone();
        let tangle = MsTangle::<N::Backend>::new(storage);

        node.register_resource(tangle);

        let tangle = node.resource::<MsTangle::<N::Backend>>().clone();
        node.spawn::<Self, _, _>(|shutdown| async move {
            use tokio::time::interval;
            use std::time::Duration;
            use futures::StreamExt;

            let mut receiver = ShutdownStream::new(shutdown, interval(Duration::from_secs(1)));

            while receiver.next().await.is_some() {
                println!("Tangle len = {}", tangle.len());
            }
        });

        Ok(Self)
    }
}
