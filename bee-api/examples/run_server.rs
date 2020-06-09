use bee_api::server;

#[tokio::main]
async fn main() {
    bee_tangle::init();

    server::run(server::SERVER_ADDRESS.parse().unwrap()).await;
}

