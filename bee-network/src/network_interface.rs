use std::{
    sync::Arc,
    collections::HashMap,
};

use futures::{channel::mpsc, lock::Mutex};

use async_std::{
    net::SocketAddr,
    task
};

type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;

use crate::bind::TcpServerConfig;
use crate::bind;
use crate::add_peer;
use crate::process_stream;
use crate::read_task_broker;
use crate::assign_message;
use crate::write_task_broker;
use crate::remove_peer;

use crate::message::MessageToSend;
use crate::add_peer::TcpClientConfig;
use crate::message::ReceivedMessage;
use crate::graceful_shutdown;
use std::io::Error;

pub async fn new (

    server_config: TcpServerConfig,
    peers_to_add_receiver: Receiver<TcpClientConfig>,
    received_messages_sender: Sender<ReceivedMessage>,
    messages_to_send_receiver: Receiver<MessageToSend>,
    peers_to_remove_receiver: Receiver<SocketAddr>,
    graceful_shutdown_receiver: Receiver<()>,
    connected_peers_sender: Sender<SocketAddr>

    ) -> Result<(), Error> {

    // spawn bind_task
    let (bind_task_shutdown_sender, bind_task_shutdown_receiver) = mpsc::unbounded();
    let (tcp_stream_sender, tcp_stream_receiver) = mpsc::unbounded();
    let bind_task = task::spawn(bind::bind(bind_task_shutdown_receiver, server_config, tcp_stream_sender.clone()));

    // spawn add_peer_task
    let (add_peer_task_shutdown_sender, add_peer_task_shutdown_receiver) = mpsc::unbounded();
    let add_peer_task = task::spawn(add_peer::add_peer(add_peer_task_shutdown_receiver, peers_to_add_receiver, tcp_stream_sender));

    // process_stream_task
    let (read_task_sender, read_task_receiver) = mpsc::unbounded();
    let (write_task_sender, write_task_receiver) = mpsc::unbounded();
    let process_stream_task = task::spawn(process_stream::process_stream(tcp_stream_receiver, read_task_sender, write_task_sender));

    // start read_task broker
    let shutdown_handles_of_read_tasks: Arc<Mutex<HashMap<SocketAddr, Sender<()>>>> = Arc::new(Mutex::new(HashMap::new()));
    let read_task_broker_task = task::spawn(read_task_broker::read_task_broker( read_task_receiver, received_messages_sender, Arc::clone(&shutdown_handles_of_read_tasks)));

    // start assign_message
    let (assign_message_task_shutdown_sender, assign_message_task_shutdown_receiver) = mpsc::unbounded();
    let senders_of_write_tasks: Arc<Mutex<HashMap<SocketAddr, Sender<MessageToSend>>>> = Arc::new(Mutex::new(HashMap::new()));
    let assign_message_task = task::spawn(assign_message::assign_message(assign_message_task_shutdown_receiver, messages_to_send_receiver, Arc::clone(&senders_of_write_tasks)));

    // start write_task broker
    let shutdown_handles_of_write_tasks: Arc<Mutex<HashMap<SocketAddr, Sender<()>>>> = Arc::new(Mutex::new(HashMap::new()));
    let write_task_broker_task = task::spawn(write_task_broker::write_task_broker( write_task_receiver, Arc::clone(&senders_of_write_tasks),Arc::clone(&shutdown_handles_of_write_tasks), connected_peers_sender));

    // remove_peer task
    let (remove_peer_shutdown_sender, remove_peer_shutdown_receiver) = mpsc::unbounded();
    let remove_peer_task = task::spawn(remove_peer::remove_peer(remove_peer_shutdown_receiver, peers_to_remove_receiver, Arc::clone(&shutdown_handles_of_read_tasks), Arc::clone(&shutdown_handles_of_write_tasks), Arc::clone(&senders_of_write_tasks)));

    // graceful shutdown
    task::spawn(graceful_shutdown::graceful_shutdown(graceful_shutdown_receiver, bind_task_shutdown_sender, add_peer_task_shutdown_sender, assign_message_task_shutdown_sender, remove_peer_shutdown_sender));

    bind_task.await;
    add_peer_task.await;
    process_stream_task.await;
    read_task_broker_task.await;
    assign_message_task.await;
    write_task_broker_task.await;
    remove_peer_task.await;

    Ok(())

}
