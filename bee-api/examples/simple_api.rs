
use bee_api::rpc_server;

fn main() {

    let rpc_server = rpc_server();
    rpc_server.wait();

}