use bee_api::rpc_server;

fn main() {

    async_std::task::block_on(rpc_server::run());

}