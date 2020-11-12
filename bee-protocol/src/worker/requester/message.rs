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
    milestone::MilestoneIndex,
    packet::MessageRequest,
    protocol::{Protocol, Sender},
};

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_common_ext::{node::Node, worker::Worker};
use bee_message::MessageId;

use async_trait::async_trait;
use dashmap::DashMap;
use futures::{select, StreamExt};
use log::{debug, info};
use tokio::time::interval;

use std::{
    ops::Deref,
    time::{Duration, Instant},
};

const RETRY_INTERVAL_SEC: u64 = 5;

#[derive(Default)]
pub(crate) struct RequestedMessages(DashMap<MessageId, (MilestoneIndex, Instant)>);

impl Deref for RequestedMessages {
    type Target = DashMap<MessageId, (MilestoneIndex, Instant)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(crate) struct MessageRequesterWorkerEvent(pub(crate) MessageId, pub(crate) MilestoneIndex);

pub(crate) struct MessageRequesterWorker {
    pub(crate) tx: flume::Sender<MessageRequesterWorkerEvent>,
}

async fn process_request(
    requested_messages: &RequestedMessages,
    message_id: MessageId,
    index: MilestoneIndex,
    counter: &mut usize,
) {
    if requested_messages.contains_key(&message_id) {
        return;
    }

    if process_request_unchecked(message_id, index, counter).await {
        requested_messages.insert(message_id, (index, Instant::now()));
    }
}

/// Return `true` if the message was requested.
async fn process_request_unchecked(message_id: MessageId, index: MilestoneIndex, counter: &mut usize) -> bool {
    if Protocol::get().peer_manager.peers.is_empty() {
        return false;
    }

    let guard = Protocol::get().peer_manager.peers_keys.read().await;

    for _ in 0..guard.len() {
        let epid = &guard[*counter % guard.len()];

        *counter += 1;

        if let Some(peer) = Protocol::get().peer_manager.peers.get(epid) {
            if peer.has_data(index) {
                Sender::<MessageRequest>::send(epid, MessageRequest::new(message_id.as_ref()));
                return true;
            }
        }
    }

    for _ in 0..guard.len() {
        let epid = &guard[*counter % guard.len()];

        *counter += 1;

        if let Some(peer) = Protocol::get().peer_manager.peers.get(epid) {
            if peer.maybe_has_data(index) {
                Sender::<MessageRequest>::send(epid, MessageRequest::new(message_id.as_ref()));
                return true;
            }
        }
    }

    false
}

async fn retry_requests(requested_messages: &RequestedMessages, counter: &mut usize) {
    let mut retry_counts: usize = 0;

    for mut message in requested_messages.iter_mut() {
        let (message_id, (index, instant)) = message.pair_mut();
        let now = Instant::now();
        if (now - *instant).as_secs() > RETRY_INTERVAL_SEC
            && process_request_unchecked(*message_id, *index, counter).await
        {
            *instant = now;
            retry_counts += 1;
        }
    }

    if retry_counts > 0 {
        debug!("Retried {} messages.", retry_counts);
    }
}

#[async_trait]
impl<N: Node> Worker<N> for MessageRequesterWorker {
    type Config = ();
    type Error = WorkerError;

    async fn start(node: &mut N, _config: Self::Config) -> Result<Self, Self::Error> {
        let (tx, rx) = flume::unbounded();

        let requested_messages: RequestedMessages = Default::default();
        node.register_resource(requested_messages);
        let requested_messages = node.resource::<RequestedMessages>();

        node.spawn::<Self, _, _>(|shutdown| async move {
            info!("Running.");

            let mut receiver = ShutdownStream::new(shutdown, rx.into_stream());

            let mut counter: usize = 0;
            let mut timeouts = interval(Duration::from_secs(RETRY_INTERVAL_SEC)).fuse();

            loop {
                select! {
                    _ = timeouts.next() => retry_requests(&*requested_messages,&mut counter).await,
                    entry = receiver.next() => match entry {
                        Some(MessageRequesterWorkerEvent(message_id, index)) =>
                            process_request(&*requested_messages, message_id, index, &mut counter).await,
                        None => break,
                    },
                }
            }

            info!("Stopped.");
        });

        Ok(Self { tx })
    }

    // TODO stop + remove_resource
}
