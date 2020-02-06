use hex;
use blake2::{Blake2b, Digest};

use bee_network::message::Message;
use bee_network::message::TestMessage;
use bee_network::message::MessageType;
use bee_network::message::ReceivedMessage;
use bee_network::message::MessageToSend;
use bee_network::network_interface;
use bee_network::network_interface::TcpClientConfig;

use std::{
    sync::Arc,
    collections::{HashSet, HashMap},
};

use futures::{channel::mpsc, SinkExt, lock::Mutex};

use async_std::{
    net::SocketAddr,
    prelude::*,
    task
};
use async_std::task::JoinHandle;
use std::io::Error;
use bee_network::network_interface::TcpServerConfig;

type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;

pub struct Node {
    transactions: Arc<Mutex<HashMap<String, TestMessage>>>,
    peers_to_add_sender: Sender<TcpClientConfig>,
    messages_to_send_sender: Sender<MessageToSend>,
    peers_to_remove_sender: Sender<SocketAddr>,
    graceful_shutdown_sender: Sender<()>,
    network_interface_handle: Arc<Mutex<JoinHandle<Result<(), Error>>>>
}

impl Node {

    pub async fn new(server_config: TcpServerConfig) -> Self {

        let transactions: Arc<Mutex<HashMap<String, TestMessage>>> = Arc::new(Mutex::new(HashMap::new()));
        let node_address = server_config.address.clone();

        let (peers_to_add_sender, peers_to_add_receiver) = mpsc::unbounded();
        let (received_messages_sender, received_messages_receiver) = mpsc::unbounded();
        let (messages_to_send_sender, messages_to_send_receiver) = mpsc::unbounded();
        let (peers_to_remove_sender, peers_to_remove_receiver) = mpsc::unbounded();
        let (graceful_shutdown_sender, graceful_shutdown_receiver) = mpsc::unbounded();
        let (connected_peers_sender, connected_peers_receiver) = mpsc::unbounded();

        let network_interface_handle = task::spawn(network_interface::bind(server_config, peers_to_add_receiver, received_messages_sender,messages_to_send_receiver, peers_to_remove_receiver, graceful_shutdown_receiver, connected_peers_sender));
        task::spawn(simple_receiver_and_gossip_logic(node_address.clone(), Arc::clone(&transactions), received_messages_receiver, messages_to_send_sender.clone()));
        task::spawn(on_new_connected_peer(connected_peers_receiver));

        Self {
            transactions,
            peers_to_add_sender,
            messages_to_send_sender,
            peers_to_remove_sender,
            graceful_shutdown_sender,
            network_interface_handle: Arc::new(Mutex::new(network_interface_handle))
        }

    }

    pub async fn send_message(&mut self, message: MessageToSend) {
        self.messages_to_send_sender.send(message).await.unwrap();
    }

    pub async fn add_peer(&mut self, client_config: TcpClientConfig) {
        self.peers_to_add_sender.send(client_config).await.unwrap();
    }

    pub async fn remove_peer(&mut self, address: SocketAddr) {
        self.peers_to_remove_sender.send(address).await.unwrap();
    }

    pub async fn graceful_shutdown(&mut self) {
        self.graceful_shutdown_sender.send(()).await.unwrap();
        (&mut *self.network_interface_handle.lock().await).await.unwrap();
    }

    pub async fn has_transaction_received(&self, hash: String) -> bool {
        let transactions: &HashMap<String, TestMessage>=  &*self.transactions.lock().await;
        if transactions.contains_key(&hash) {
            return true;
        }
        false
    }

}

pub async fn simple_receiver_and_gossip_logic(node_address: String, transactions: Arc<Mutex<HashMap<String, TestMessage>>>, mut message_receiver: Receiver<ReceivedMessage>, mut message_sender: Sender<MessageToSend>) {

    while let Some(message) = message_receiver.next().await {

        let message: ReceivedMessage = message;

        let message = match message.msg {
            MessageType::Test(x) => x
        };

        let mut hasher = Blake2b::new();
        hasher.input(Message::bytes(&message));
        let hash = hex::encode(hasher.result());

        let transactions: &mut HashMap<String, TestMessage> = &mut *transactions.lock().await;

        if transactions.contains_key(&hash) {
            continue;
        }

        transactions.insert(hash, message.clone());

        println!("{} received message: {}", node_address, message);

        // send message to *all* peers
        let message_to_send = MessageToSend { to: HashSet::new(), msg: MessageType::Test(message)};
        message_sender.send(message_to_send).await.unwrap();

    }

}

pub async fn on_new_connected_peer(mut connected_peers_receiver: Receiver<SocketAddr>) {

    while let Some(_address) = connected_peers_receiver.next().await {

       // New node connected

    }

}