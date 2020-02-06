use std::str::FromStr;
use blake2::{Blake2b, Digest};
use std::collections::HashSet;
use std::time::Duration;

use async_std::task;
use async_std::net::SocketAddr;

use crate::node::Node;

use bee_network::message::MessageType;
use bee_network::message::Message;
use bee_network::message::TestMessage;
use bee_network::message::MessageToSend;
use bee_network::network_interface::TcpClientConfig;
use bee_network::network_interface::TcpServerConfig;

mod node;

fn main () {
    task::block_on(async_test_receive_message());
}

async fn async_test_receive_message() {

    // create node instances
    let mut node_1:Node = Node::new(TcpServerConfig { address: String::from("127.0.0.1:8081") }).await;
    let mut node_2:Node = Node::new(TcpServerConfig { address: String::from("127.0.0.1:8082") }).await;
    let mut node_3:Node = Node::new(TcpServerConfig { address: String::from("127.0.0.1:8083") }).await;
    let mut node_4:Node = Node::new(TcpServerConfig { address: String::from("127.0.0.1:8084") }).await;

    // peer node 1 with node 2
    node_1.add_peer(TcpClientConfig{address: String::from("127.0.0.1:8082") }).await;

    // peer node 2 with node 3
    node_2.add_peer(TcpClientConfig{address: String::from("127.0.0.1:8083") }).await;

    // peer node 2 with node 4
    node_2.add_peer(TcpClientConfig{address: String::from("127.0.0.1:8084") }).await;

    delay().await;

    // send a message from node 1 to node 2
    let message = TestMessage::new("Hello World!".to_string());
    let message_to_send = MessageToSend { to: HashSet::new(), msg: MessageType::Test(message.clone())};
    node_1.send_message(message_to_send).await;
    delay().await;

    assert_eq!(true, node_2.has_transaction_received(hash(Message::bytes(&message))).await);
    assert_eq!(true, node_3.has_transaction_received(hash(Message::bytes(&message))).await); // Also node 3 should have received it; node_2 should have gossiped it
    assert_eq!(true, node_4.has_transaction_received(hash(Message::bytes(&message))).await); // Also node 4 should have received it since its connected with node 2

    // remove peer 4
    println!("Removing node 4...");
    node_2.remove_peer( SocketAddr::from_str("127.0.0.1:8084").unwrap()).await;

    let message = TestMessage::new("Another message!".to_string());
    let message_to_send = MessageToSend { to: HashSet::new(), msg: MessageType::Test(message.clone())};
    node_1.send_message(message_to_send).await;
    delay().await;

    assert_eq!(true, node_2.has_transaction_received(hash(Message::bytes(&message))).await);
    assert_eq!(true, node_3.has_transaction_received(hash(Message::bytes(&message))).await);
    assert_eq!(false, node_4.has_transaction_received(hash(Message::bytes(&message))).await);

    node_1.graceful_shutdown().await;
    node_2.graceful_shutdown().await;
    node_3.graceful_shutdown().await;
    node_4.graceful_shutdown().await;

}

fn hash(bytes: &[u8]) -> String {
    let mut hasher = Blake2b::new();
    hasher.input(bytes);
    let hash = hasher.result();
    hex::encode(hash)
}

async fn delay()  {
    async_std::task::sleep(Duration::from_millis(1000u64)).await;
}