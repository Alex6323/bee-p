mod logger;
mod node;

use node::Node;

use bee_network::Address;

use async_std::task::block_on;

fn main() {
    logger::init(log::LevelFilter::Debug);

    let addr = block_on(Address::from_host_addr("localhost:1337")).unwrap();
    let (network, shutdown, receiver) = bee_network::init(addr);

    let mut node = Node::new(network, shutdown, receiver);

    block_on(node.init());

    block_on(node.run());

    // task::spawn(notification_handler(receiver));
    //
    // block_on(node.init());
    //
    // // NOTE: all the node business logic has to go inside of the following scope!!!
    // {
    //     // For example: spamming the network
    //     std::thread::spawn(|| spam(network, msg, 50, 1000));
    // }
    //
    // block_on(node.shutdown());
}
