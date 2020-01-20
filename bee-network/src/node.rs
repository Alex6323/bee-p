use std::collections::HashMap;
use crate::tcp_server::TcpServerConfig;
use crate::tcp_server::TcpClientConfig;
use crate::mpmc;
use std::sync::Arc;
use crate::tcp_server::ReceivedMessage;
use crate::message::MessageType;
use hex;

use async_std::{
    task,
    sync::Mutex
};

use blake2::{Blake2b, Digest};

use crate::message::Message;
use crate::message::TestMessage;
use crate::network_interface;
use crate::tcp_server::MessageToSend;
use futures::{channel::mpsc, SinkExt};
use std::collections::HashSet;

pub struct Node {
    transactions: Arc<Mutex<HashMap<String, TestMessage>>>,
    messages_to_send_sender: mpmc::Sender<MessageToSend>,
    peers_to_add_sender: mpsc::UnboundedSender<TcpClientConfig>,
    shutdown_sender: mpmc::Sender<()>,
}

impl Node {

    pub async fn new(server_config: TcpServerConfig) -> Self {

        let transactions: Arc<Mutex<HashMap<String, TestMessage>>> = Arc::new(Mutex::new(HashMap::new()));
        let network_access_handles = network_interface::new(server_config).await;

        let messages_to_send_sender = network_access_handles.messages_to_send_sender;
        let received_messages_receiver = network_access_handles.received_messages_receiver;

        task::spawn(simple_receiver_and_gossip_logic(Arc::clone(&transactions), received_messages_receiver, messages_to_send_sender.clone()));

        Self {
            transactions,
            messages_to_send_sender,
            peers_to_add_sender: network_access_handles.peers_to_add_sender,
            shutdown_sender: network_access_handles.shutdown_sender,
        }

    }

    pub async fn send_message(&mut self, message: MessageToSend) {
        self.messages_to_send_sender.send(message).await.unwrap();
    }

    pub async fn add_peer(&mut self, client_config: TcpClientConfig) {
        self.peers_to_add_sender.send(client_config).await.unwrap();
    }

    pub async fn shutdown(&mut self) {
        self.shutdown_sender.send(()).await.unwrap();
        println!("Shutdown node...");
    }

    pub async fn has_received(&self, hash: String) -> bool {
        let transactions: &HashMap<String, TestMessage>=  &*self.transactions.lock().await;
        if transactions.contains_key(&hash) {
            return true;
        }
        false
    }

}

pub async fn simple_receiver_and_gossip_logic(transactions: Arc<Mutex<HashMap<String, TestMessage>>>, mut message_receiver: mpmc::Receiver<ReceivedMessage>, mut message_sender: mpmc::Sender<MessageToSend>) {

    while let Some(message) = message_receiver.next().await {

        let message: ReceivedMessage = message;

        let message = match message.msg {
            MessageType::Test(x) => x
        };

        let mut hasher = Blake2b::new();
        hasher.input(Message::bytes(&message));
        let hash = hex::encode(hasher.result());

        let transactions: &mut HashMap<String, TestMessage> = &mut *transactions.lock().await;

        if !transactions.contains_key(&hash) {
            transactions.insert(hash, message.clone());
        }

        // send message to *all* peers
        let message_to_send = MessageToSend { to: HashSet::new(), msg: MessageType::Test(message)};
        message_sender.send(message_to_send).await.unwrap();

    }

}

