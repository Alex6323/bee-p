use crate::api::{ApiImpl, Api};

use bee_ternary::{TryteBuf, T1B1Buf};
use bee_transaction::{Hash, BundledTransactionField, Payload};

use jsonrpsee::common;
use std::net::SocketAddr;

pub const RPC_SERVER_ADDRESS: &str = "127.0.0.1:8000";

jsonrpsee::rpc_api! {
    RequestType {
        fn transactions_by_hash(hashes: Vec<String>) -> Vec<Payload>; // change return value to Vec<Option<BundledTransaction>>
    }
}

pub async fn run(listen_addr: SocketAddr) {
    let transport_server = jsonrpsee::transport::http::HttpTransportServer::bind(&listen_addr)
        .await
        .unwrap();
    let mut server = jsonrpsee::raw::RawServer::new(transport_server);

    'outer: while let Ok(request) = RequestType::next_request(&mut server).await {
        match request {
            RequestType::TransactionsByHash { respond, hashes } => {

                // Deserialize hashes of request
                let hashes = {
                    let mut ret = Vec::new();

                    for tryte_string in hashes {
                        let hash_buf = TryteBuf::try_from_str(&tryte_string)
                            .unwrap()
                            .as_trits()
                            .encode::<T1B1Buf>();

                        let hash_result = Hash::try_from_inner(hash_buf);

                        match hash_result {
                            Ok(hash) => ret.push(hash),
                            Err(_) => {
                                respond.err(common::Error::invalid_params("Invalid params provided!")).await;
                                continue 'outer;
                            }
                        }
                    }

                    ret
                };

                let mut response = Vec::new();
                for tx_ref in ApiImpl::transactions_by_hash(&hashes) {
                    match tx_ref {
                        Some(tx_ref) => {
                            // Pushing only the transaction payload to the response
                            response.push(tx_ref.payload().clone());
                        }
                        None => response.push(Payload::zeros())
                    }
                }

                respond.ok(response).await;

            }
        }
    }
}