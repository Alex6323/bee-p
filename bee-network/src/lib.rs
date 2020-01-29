use std::io::Error;
use std::time::Duration;
use async_std::task;

use crate::message::{Message};
use crate::tcp_server::TcpServerConfig;
use crate::tcp_server::TcpClientConfig;
use crate::tcp_server::MessageToSend;
use crate::node::Node;

use blake2::{Blake2b, Digest};
use crate::message::TestMessage;
use std::collections::HashSet;
use crate::message::MessageType;

mod message;
mod tcp_server;
mod mpmc;
mod network_interface;
mod node;

#[test]
fn test_receive_message() {
    task::block_on(async_test_receive_message());
}

async fn async_test_receive_message() -> Result<(), Error> {

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

    assert_eq!(true, node_2.has_received(hash(Message::bytes(&message))).await);
    assert_eq!(true, node_3.has_received(hash(Message::bytes(&message))).await); // Also node 3 should have received it; node_2 should have gossiped it
    assert_eq!(true, node_4.has_received(hash(Message::bytes(&message))).await); // Also node 4 should have received it since its connected with node 2

    node_1.shutdown().await;
    node_2.shutdown().await;
    node_3.shutdown().await;
    node_4.shutdown().await;

    delay().await;

    Ok(())

}

fn hash(bytes: &[u8]) -> String {
    let mut hasher = Blake2b::new();
    hasher.input(bytes);
    let hash = hasher.result();
    hex::encode(hash)
}

async fn delay()  {
    async_std::task::sleep(Duration::from_millis(3000u64)).await;
}