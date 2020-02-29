use bee_common::logger;
use bee_protocol::Node;
use netzwerk::ConfigBuilder;

use async_std::task::block_on;

fn main() {
    // let args = Args::from_args();
    // let config = args.make_config();
    let config = ConfigBuilder::new()
        .with_peer("tcp://127.0.0.1:15600")
        .build();

    logger::init(log::LevelFilter::Debug);

    let (network, shutdown, receiver) = netzwerk::init(config.clone());

    let mut node = Node::new(config, network, shutdown, receiver);

    block_on(node.init());

    node.start();

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
