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

use crate::{tangle::MsTangle, worker::TangleWorker};

use bee_common::shutdown_stream::ShutdownStream;
use bee_common_ext::{node::Node, worker::Worker};
use bee_message::MessageId;

use async_trait::async_trait;
use futures::stream::StreamExt;
use log::info;

use std::{any::TypeId, convert::Infallible};

pub(crate) struct MessageValidatorWorkerEvent(pub(crate) MessageId);

pub(crate) struct MessageValidatorWorker {
    pub(crate) tx: flume::Sender<MessageValidatorWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for MessageValidatorWorker {
    type Config = ();
    type Error = Infallible;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<TangleWorker>()].leak()
    }

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = flume::unbounded();

        let _tangle = node.resource::<MsTangle<N::Backend>>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx.into_stream());

            while let Some(MessageValidatorWorkerEvent(_hash)) = receiver.next().await {
                // if let Ok(bundle) = builder.validate() {
                //     tangle.update_metadata(&hash, |metadata| {
                //         metadata.flags_mut().set_valid(true);
                //     });
                //     tangle.insert_tip(hash, *bundle.parent1(), *bundle.parent2()).await;
                // }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }
}
