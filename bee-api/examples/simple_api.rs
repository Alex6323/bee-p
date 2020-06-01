use bee_api::rpc_server;
use bee_api::rpc_server::RPC_SERVER_ADDRESS;


fn main() {

    async_std::task::block_on(rpc_server::run(RPC_SERVER_ADDRESS.parse().unwrap()));

}