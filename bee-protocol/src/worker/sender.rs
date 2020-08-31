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
    message::{
        tlv_into_bytes, Heartbeat, Message, MilestoneRequest, Transaction as TransactionMessage, TransactionRequest,
    },
    peer::HandshakedPeer,
    protocol::Protocol,
};

use bee_common::shutdown_stream::ShutdownStream;
use bee_network::{Command::SendMessage, EndpointId, Network};

use futures::{
    channel::mpsc,
    stream::{Fuse, StreamExt},
};
use log::warn;

use std::sync::Arc;

type Receiver<M> = ShutdownStream<Fuse<mpsc::UnboundedReceiver<M>>>;

pub(crate) struct SenderWorker<M: Message> {
    receiver: Receiver<M>,
}

macro_rules! implement_sender_worker {
    ($type:ty, $sender:tt, $incrementor:tt) => {
        impl SenderWorker<$type> {
            pub(crate) async fn send(epid: &EndpointId, message: $type) {
                match Protocol::get()
                    .network
                    .clone()
                    .send(SendMessage {
                        epid: *epid,
                        bytes: tlv_into_bytes(message),
                        responder: None,
                    })
                    .await
                {
                    Ok(_) => {
                        // self.peer.metrics.$incrementor();
                        // Protocol::get().metrics.$incrementor();
                    }
                    Err(e) => {
                        warn!("Sending {} to {} failed: {:?}.", stringify!($type), epid, e);
                    }
                }
            }
        }
    };
}

implement_sender_worker!(MilestoneRequest, milestone_request, milestone_requests_sent_inc);
implement_sender_worker!(TransactionMessage, transaction, transactions_sent_inc);
implement_sender_worker!(TransactionRequest, transaction_request, transaction_requests_sent_inc);
implement_sender_worker!(Heartbeat, heartbeat, heartbeats_sent_inc);

// TODO is this really necessary ?
