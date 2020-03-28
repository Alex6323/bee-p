use crate::{
    message::{
        Heartbeat,
        MilestoneRequest,
        TransactionBroadcast,
        TransactionRequest,
    },
    peer::{
        Peer,
        PeerMetrics,
    },
    protocol::{
        HEARTBEAT_SEND_BOUND,
        MILESTONE_REQUEST_SEND_BOUND,
        TRANSACTION_BROADCAST_SEND_BOUND,
        TRANSACTION_REQUEST_SEND_BOUND,
    },
    worker::{
        SenderContext,
        SenderRegistry,
        SenderWorker,
    },
};

use bee_network::Network;

use std::sync::Arc;

use async_std::task::spawn;
use futures::channel::{
    mpsc,
    oneshot,
};

pub fn protocol_init() {
    SenderRegistry::init();
}

pub(crate) async fn protocol_add(network: Network, peer: Arc<Peer>, metrics: Arc<PeerMetrics>) {
    // SenderWorker MilestoneRequest channels
    let (milestone_request_tx, milestone_request_rx) = mpsc::channel(MILESTONE_REQUEST_SEND_BOUND);
    let (milestone_request_shutdown_tx, milestone_request_shutdown_rx) = oneshot::channel();

    // SenderWorker TransactionBroadcast channels
    let (transaction_broadcast_tx, transaction_broadcast_rx) = mpsc::channel(TRANSACTION_BROADCAST_SEND_BOUND);
    let (transaction_broadcast_shutdown_tx, transaction_broadcast_shutdown_rx) = oneshot::channel();

    // SenderWorker TransactionRequest channels
    let (transaction_request_tx, transaction_request_rx) = mpsc::channel(TRANSACTION_REQUEST_SEND_BOUND);
    let (transaction_request_shutdown_tx, transaction_request_shutdown_rx) = oneshot::channel();

    // SenderWorker Heartbeat channels
    let (heartbeat_tx, heartbeat_rx) = mpsc::channel(HEARTBEAT_SEND_BOUND);
    let (heartbeat_shutdown_tx, heartbeat_shutdown_rx) = oneshot::channel();

    let context = SenderContext::new(
        (milestone_request_tx, milestone_request_shutdown_tx),
        (transaction_broadcast_tx, transaction_broadcast_shutdown_tx),
        (transaction_request_tx, transaction_request_shutdown_tx),
        (heartbeat_tx, heartbeat_shutdown_tx),
    );
    SenderRegistry::insert(peer.epid, context).await;

    spawn(
        SenderWorker::<MilestoneRequest>::new(peer.clone(), metrics.clone(), network.clone())
            .run(milestone_request_rx, milestone_request_shutdown_rx),
    );
    spawn(
        SenderWorker::<TransactionBroadcast>::new(peer.clone(), metrics.clone(), network.clone())
            .run(transaction_broadcast_rx, transaction_broadcast_shutdown_rx),
    );
    spawn(
        SenderWorker::<TransactionRequest>::new(peer.clone(), metrics.clone(), network.clone())
            .run(transaction_request_rx, transaction_request_shutdown_rx),
    );
    spawn(
        SenderWorker::<Heartbeat>::new(peer.clone(), metrics.clone(), network.clone())
            .run(heartbeat_rx, heartbeat_shutdown_rx),
    );
}
