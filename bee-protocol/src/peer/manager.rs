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

// TODO get peer info

use crate::{
    message::{Heartbeat, MilestoneRequest, Transaction as TransactionMessage, TransactionRequest},
    peer::{HandshakedPeer, Peer},
    protocol::Protocol,
    worker::{Receiver, SenderWorker},
};

use bee_network::{Address, EndpointId, Network};

use std::sync::{Arc, Mutex};

use async_std::{sync::RwLock, task::spawn};
use dashmap::DashMap;
use futures::channel::{mpsc, oneshot};
use log::warn;

pub(crate) struct PeerManager {
    network: Network,
    pub(crate) peers: DashMap<EndpointId, Arc<Peer>>,
    pub(crate) handshaked_peers: DashMap<EndpointId, Arc<HandshakedPeer>>,
    pub(crate) handshaked_peers_keys: RwLock<Vec<EndpointId>>,
}

impl PeerManager {
    pub(crate) fn new(network: Network) -> Self {
        Self {
            network,
            peers: Default::default(),
            handshaked_peers: Default::default(),
            handshaked_peers_keys: Default::default(),
        }
    }

    pub(crate) fn add(&self, peer: Arc<Peer>) {
        self.peers.insert(peer.epid, peer);
    }

    pub(crate) async fn handshake(&self, epid: &EndpointId, address: Address) {
        if self.peers.remove(epid).is_some() {
            // TODO check if not already added

            // SenderWorker MilestoneRequest
            let (milestone_request_tx, milestone_request_rx) =
                mpsc::channel(Protocol::get().config.workers.milestone_request_send_worker_bound);
            let (milestone_request_shutdown_tx, milestone_request_shutdown_rx) = oneshot::channel();

            // SenderWorker TransactionMessage
            let (transaction_tx, transaction_rx) =
                mpsc::channel(Protocol::get().config.workers.transaction_send_worker_bound);
            let (transaction_shutdown_tx, transaction_shutdown_rx) = oneshot::channel();

            // SenderWorker TransactionRequest
            let (transaction_request_tx, transaction_request_rx) =
                mpsc::channel(Protocol::get().config.workers.transaction_request_send_worker_bound);
            let (transaction_request_shutdown_tx, transaction_request_shutdown_rx) = oneshot::channel();

            // SenderWorker Heartbeat
            let (heartbeat_tx, heartbeat_rx) =
                mpsc::channel(Protocol::get().config.workers.heartbeat_send_worker_bound);
            let (heartbeat_shutdown_tx, heartbeat_shutdown_rx) = oneshot::channel();

            let peer = Arc::new(HandshakedPeer::new(
                *epid,
                address,
                (milestone_request_tx, Mutex::new(Some(milestone_request_shutdown_tx))),
                (transaction_tx, Mutex::new(Some(transaction_shutdown_tx))),
                (
                    transaction_request_tx,
                    Mutex::new(Some(transaction_request_shutdown_tx)),
                ),
                (heartbeat_tx, Mutex::new(Some(heartbeat_shutdown_tx))),
            ));

            self.handshaked_peers.insert(*epid, peer.clone());
            self.handshaked_peers_keys.write().await.push(*epid);

            spawn(
                SenderWorker::<MilestoneRequest>::new(
                    self.network.clone(),
                    peer.clone(),
                    Receiver::new(milestone_request_rx, milestone_request_shutdown_rx),
                )
                .run(),
            );
            spawn(
                SenderWorker::<TransactionMessage>::new(
                    self.network.clone(),
                    peer.clone(),
                    Receiver::new(transaction_rx, transaction_shutdown_rx),
                )
                .run(),
            );
            spawn(
                SenderWorker::<TransactionRequest>::new(
                    self.network.clone(),
                    peer.clone(),
                    Receiver::new(transaction_request_rx, transaction_request_shutdown_rx),
                )
                .run(),
            );
            spawn(
                SenderWorker::<Heartbeat>::new(
                    self.network.clone(),
                    peer,
                    Receiver::new(heartbeat_rx, heartbeat_shutdown_rx),
                )
                .run(),
            );

            if self.handshaked_peers.len() == 1 {
                Protocol::request_milestone_initial();
            }
        }
    }

    pub(crate) async fn remove(&self, epid: &EndpointId) {
        // TODO both ?
        self.peers.remove(epid);

        self.handshaked_peers_keys.write().await.retain(|e| e != epid);

        if let Some((_, peer)) = self.handshaked_peers.remove(epid) {
            if let Ok(mut shutdown) = peer.milestone_request.1.lock() {
                if let Some(shutdown) = shutdown.take() {
                    if let Err(e) = shutdown.send(()) {
                        warn!("Shutting down TransactionWorker failed: {:?}.", e);
                    }
                }
            }
        }

        // TODO

        // if let Err(_) = peer.milestone_request.1.send(()) {
        //     warn!("Shutting down MilestoneRequest SenderWorker failed.");
        // }
        // if let Err(_) = peer.transaction.1.send(()) {
        //     warn!("Shutting down TransactionMessage SenderWorker failed.");
        // }
        // if let Err(_) = peer.transaction_request.1.send(()) {
        //     warn!("Shutting down TransactionRequest SenderWorker failed.");
        // }
        // if let Err(_) = peer.heartbeat.1.send(()) {
        //     warn!("Shutting down Heartbeat SenderWorker failed.");
        // }
    }
}
