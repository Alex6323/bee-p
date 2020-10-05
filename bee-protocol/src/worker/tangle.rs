use crate::MilestoneIndex;

use async_trait::async_trait;
use bee_common::shutdown_stream::ShutdownStream;
use bee_common_ext::{node::Node, worker::Worker};
use bee_snapshot::metadata::SnapshotMetadata;
use log::info;
use std::{any::TypeId, convert::Infallible};

use crate::{tangle::MsTangle, worker::storage::StorageWorker};

pub struct TangleWorker;

#[async_trait]
impl<N: Node> Worker<N> for TangleWorker {
    type Config = SnapshotMetadata;
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        Box::leak(Box::from(vec![TypeId::of::<StorageWorker>()]))
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let storage = node.storage().clone();
        let tangle = MsTangle::<N::Backend>::new(storage);

        node.register_resource(tangle);

        let tangle = node.resource::<MsTangle<N::Backend>>().clone();

        tangle.update_latest_solid_milestone_index(config.index().into());
        tangle.update_latest_milestone_index(config.index().into());
        tangle.update_snapshot_index(config.index().into());
        tangle.update_pruning_index(config.index().into());

        for (hash, index) in config.solid_entry_points() {
            tangle.add_solid_entry_point(*hash, MilestoneIndex(*index));
        }
        for _seen_milestone in config.seen_milestones() {
            // TODO request ?
        }

        node.spawn::<Self, _, _>(|shutdown| async move {
            use futures::StreamExt;
            use std::time::Duration;
            use tokio::time::interval;

            let mut receiver = ShutdownStream::new(shutdown, interval(Duration::from_secs(1)));

            while receiver.next().await.is_some() {
                println!("Tangle len = {}", tangle.len());
            }
        });

        Ok(Self)
    }
}
