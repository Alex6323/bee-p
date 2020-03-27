use crate::{
    message::{
        TransactionBroadcast,
        TransactionRequest,
    },
    worker::SenderWorker,
};

use bee_network::EndpointId;

pub async fn send_transaction(epid: EndpointId, transaction: &[u8]) {
    SenderWorker::<TransactionBroadcast>::send(&epid, TransactionBroadcast::new(transaction)).await;
}

pub async fn broadcast_transaction(transaction: &[u8]) {
    SenderWorker::<TransactionBroadcast>::broadcast(TransactionBroadcast::new(transaction)).await;
}

//  TODO constant

pub async fn send_transaction_request(epid: EndpointId, hash: [u8; 49]) {
    SenderWorker::<TransactionRequest>::send(&epid, TransactionRequest::new(hash)).await;
}

pub async fn broadcast_transaction_request(hash: [u8; 49]) {
    SenderWorker::<TransactionRequest>::broadcast(TransactionRequest::new(hash)).await;
}
