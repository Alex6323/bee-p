use bee_common_ext::{
    worker::Worker,
    node::Node,
};
use bee_common::shutdown_stream::ShutdownStream;
use bee_storage::storage::Backend;
use async_trait::async_trait;
use log::info;
use std::{
    any::Any,
    error,
    fmt,
};

#[derive(Debug)]
pub struct Error(Box<dyn error::Error>);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl error::Error for Error {}

pub struct StorageWorker;

#[async_trait]
impl<N: Node> Worker<N> for StorageWorker {
    type Config = <N::Backend as Backend>::Config;
    type Error = Error;

    async fn start(
        node: &mut N,
        config: Self::Config,
    ) -> Result<Self, Self::Error> {
        info!("Starting Tangle worker...");

        let backend = N::Backend::start(config).await.map_err(Error)?;

        node.register_resource(backend);

        Ok(Self)
    }
}
