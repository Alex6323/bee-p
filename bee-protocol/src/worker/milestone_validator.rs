// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use crate::{
    config::ProtocolConfig,
    event::{LatestMilestoneChanged, LatestSolidMilestoneChanged},
    milestone::{Milestone, MilestoneIndex},
    protocol::Protocol,
    tangle::MsTangle,
    worker::{
        MilestoneConeUpdaterWorker, MilestoneConeUpdaterWorkerEvent, MilestoneRequesterWorker,
        MilestoneSolidifierWorker, MilestoneSolidifierWorkerEvent, RequestedMilestones, TangleWorker,
    },
};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_common_ext::{node::Node, worker::Worker};
use bee_message::MessageId;

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::{debug, error, info};

use std::any::TypeId;

#[derive(Debug)]
pub(crate) enum MilestoneValidatorWorkerError {
    UnknownMessage,
}

pub(crate) struct MilestoneValidatorWorkerEvent(pub(crate) MessageId);

pub(crate) struct MilestoneValidatorWorker {
    pub(crate) tx: flume::Sender<MilestoneValidatorWorkerEvent>,
}

async fn validate<N: Node>(
    tangle: &MsTangle<N::Backend>,
    _config: &ProtocolConfig,
    message_id: MessageId,
) -> Result<Milestone, MilestoneValidatorWorkerError>
where
    N: Node,
{
    let _message = tangle
        .get(&message_id)
        .await
        .ok_or(MilestoneValidatorWorkerError::UnknownMessage)?;

    // TODO complete
    Ok(Milestone {
        message_id,
        index: MilestoneIndex(0),
    })
}

#[async_trait]
impl<N> Worker<N> for MilestoneValidatorWorker
where
    N: Node,
{
    type Config = ProtocolConfig;
    type Error = WorkerError;

    fn dependencies() -> &'static [TypeId] {
        vec![
            TypeId::of::<MilestoneSolidifierWorker>(),
            TypeId::of::<MilestoneConeUpdaterWorker>(),
            TypeId::of::<TangleWorker>(),
            TypeId::of::<MilestoneRequesterWorker>(),
        ]
        .leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = flume::unbounded();
        let milestone_solidifier = node.worker::<MilestoneSolidifierWorker>().unwrap().tx.clone();
        let milestone_cone_updater = node.worker::<MilestoneConeUpdaterWorker>().unwrap().tx.clone();

        let tangle = node.resource::<MsTangle<N::Backend>>();
        let requested_milestones = node.resource::<RequestedMilestones>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx.into_stream());

            while let Some(MilestoneValidatorWorkerEvent(message_id)) = receiver.next().await {
                if let Some(meta) = tangle.get_metadata(&message_id) {
                    if meta.flags().is_milestone() {
                        continue;
                    }
                    match validate::<N>(&tangle, &config, message_id).await {
                        Ok(milestone) => {
                            tangle.add_milestone(milestone.index, milestone.message_id);

                            // This is possibly not sufficient as there is no guarantee a milestone has been
                            // solidified before being validated, we then also need
                            // to check when a milestone gets solidified if it's
                            // already vadidated.
                            if meta.flags().is_solid() {
                                Protocol::get()
                                    .bus
                                    .dispatch(LatestSolidMilestoneChanged(milestone.clone()));
                                if let Err(e) =
                                    milestone_cone_updater.send(MilestoneConeUpdaterWorkerEvent(milestone.clone()))
                                {
                                    error!("Sending message id to milestone validation failed: {:?}.", e);
                                }
                            }

                            if milestone.index > tangle.get_latest_milestone_index() {
                                Protocol::get().bus.dispatch(LatestMilestoneChanged(milestone.clone()));
                            }

                            if requested_milestones.remove(&milestone.index).is_some() {
                                tangle.update_metadata(&milestone.message_id, |meta| {
                                    meta.flags_mut().set_requested(true)
                                });

                                if let Err(e) =
                                    milestone_solidifier.send(MilestoneSolidifierWorkerEvent(milestone.index))
                                {
                                    error!("Sending solidification event failed: {}", e);
                                }
                            }
                        }
                        Err(e) => debug!("Invalid milestone message: {:?}.", e),
                    }
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
