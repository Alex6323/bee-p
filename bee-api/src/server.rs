
use crate::routes;

use serde::de::DeserializeOwned;
use warp::{Filter, Rejection};

use std::net::SocketAddr;

pub const SERVER_ADDRESS: &str = "127.0.0.1:3030";

pub async fn run(addr: SocketAddr) {

    let tx_by_hash = warp::post()
        .and(warp::path("v1"))
        .and(warp::path("transaction"))
        .and(warp::path::end())
        .and(json_body())
        .and_then(routes::transaction_by_hash);

    let routes = tx_by_hash;

    warp::serve(routes)
        .run(addr)
        .await;

}

fn json_body<T: DeserializeOwned + Send>() -> impl Filter<Extract = (T,), Error = Rejection> + Copy {
    warp::body::content_length_limit(1024 * 32).and(warp::body::json())
}