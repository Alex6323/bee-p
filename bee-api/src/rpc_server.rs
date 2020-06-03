use crate::api::{ApiImpl, Api};

use bee_ternary::{TryteBuf, T1B1Buf, TritBuf, Tryte};
use bee_transaction::{Hash, BundledTransactionField, BundledTransaction};

use log::*;
use jsonrpsee::common;
use std::net::SocketAddr;

pub const RPC_SERVER_ADDRESS: &str = "127.0.0.1:8000";

jsonrpsee::rpc_api! {
    RequestType {
        fn echo(msg: String) -> String; // e.g. curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0", "method": "echo", "params": {"msg": "Hello Bee"}, "id": 1}' 127.0.0.1:8000
        fn transaction_by_hash(hash: String) -> String;
    }
}

pub async fn run(listen_addr: SocketAddr) {

    info!("Starting RPC server...");

    let transport_server = jsonrpsee::transport::http::HttpTransportServer::bind(&listen_addr)
        .await
        .unwrap();
    let mut server = jsonrpsee::raw::RawServer::new(transport_server);

    while let Ok(request) = RequestType::next_request(&mut server).await {
        match request {

            RequestType::Echo { respond, msg } => {
                respond.ok(msg).await;
            }

            RequestType::TransactionByHash { respond, hash } => {

                // deserialize provided hash
                let hash_result = Hash::try_from_inner(deserialize_tryte_str(&hash));

                match hash_result {
                    Ok(hash) => {
                        match ApiImpl::transaction_by_hash(&hash) {
                            Some(tx_ref) => {
                                let mut trits = TritBuf::<T1B1Buf>::zeros(BundledTransaction::trit_len());
                                tx_ref.into_trits_allocated(&mut trits);
                                let response = trits
                                    .chunks(3)
                                    .map(|trits| char::from(Tryte::from_trits([trits.get(0).unwrap(), trits.get(1).unwrap(), trits.get(2).unwrap()])))
                                    .collect::<String>();
                                respond.ok(response).await;
                            }
                            None => {
                                let response = String::from("Transaction not found!");
                                respond.ok(response).await;
                            }
                        }
                    }
                    Err(_e) => {
                        respond.err(common::Error::parse_error()).await;
                    }
                }

            }

        }
    }
}

fn deserialize_tryte_str(tryte_str: &str) -> TritBuf {
    TryteBuf::try_from_str(tryte_str)
        .unwrap()
        .as_trits()
        .encode::<T1B1Buf>()
}