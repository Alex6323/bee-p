use bee_common::logger;

use std::{collections::HashMap, sync::Arc};

use futures::{channel::mpsc, lock::Mutex, select, FutureExt, SinkExt};

use async_std::{net::SocketAddr, task};

use crate::add_peer;
use crate::assign_message;
use crate::process_stream;
use crate::read_task_broker;
use crate::remove_peer;
use crate::write_task_broker;

use crate::graceful_shutdown;
use crate::message::{Message, MessageReader, MessageToSend, ReceivedMessage};
use std::io::Error;

pub type Sender<T> = mpsc::UnboundedSender<T>;
pub type Receiver<T> = mpsc::UnboundedReceiver<T>;

use async_std::{net::TcpListener, prelude::*};

use std::ops::Deref;

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    mpsc::unbounded()
}

pub async fn bind<R>(
    server_config: TcpServerConfig,
    peers_to_add_receiver: Receiver<TcpClientConfig>,
    received_messages_sender: Sender<ReceivedMessage<R::MessageType>>,
    messages_to_send_receiver: Receiver<MessageToSend<R::MessageType>>,
    peers_to_remove_receiver: Receiver<SocketAddr>,
    graceful_shutdown_receiver: Receiver<()>,
    connected_peers_sender: Sender<SocketAddr>,
) -> Result<(), Error>
where
    R: MessageReader + 'static,
    <R as MessageReader>::MessageType: Deref<Target = Message<Error = R::Error>>
        + std::clone::Clone
        + std::marker::Send
        + std::marker::Sync,
{
    // bind server
    let listener = TcpListener::bind(server_config.address.clone()).await?;
    let (bind_task_shutdown_sender, mut bind_task_shutdown_receiver) = mpsc::unbounded();
    let (mut tcp_stream_sender, tcp_stream_receiver) = mpsc::unbounded();
    logger::info(&format!(
        "Bee is listening for TCP packets at: {:?}",
        &server_config.address
    ));

    // spawn add_peer_task
    let (add_peer_task_shutdown_sender, add_peer_task_shutdown_receiver) = mpsc::unbounded();
    let add_peer_task = task::spawn(add_peer::add_peer(
        add_peer_task_shutdown_receiver,
        peers_to_add_receiver,
        tcp_stream_sender.clone(),
    ));

    // process_stream_task
    let (read_task_sender, read_task_receiver) = mpsc::unbounded();
    let (write_task_sender, write_task_receiver) = mpsc::unbounded();
    let process_stream_task = task::spawn(process_stream::process_stream(
        tcp_stream_receiver,
        read_task_sender,
        write_task_sender,
    ));

    // start read_task broker
    let shutdown_handles_of_read_tasks: Arc<Mutex<HashMap<SocketAddr, Sender<()>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let read_task_broker_task = task::spawn(read_task_broker::read_task_broker::<R>(
        read_task_receiver,
        received_messages_sender,
        Arc::clone(&shutdown_handles_of_read_tasks),
    ));

    // start assign_message
    let (assign_message_task_shutdown_sender, assign_message_task_shutdown_receiver) =
        mpsc::unbounded();
    let senders_of_write_tasks: Arc<
        Mutex<HashMap<SocketAddr, Sender<MessageToSend<R::MessageType>>>>,
    > = Arc::new(Mutex::new(HashMap::new()));
    let assign_message_task = task::spawn(assign_message::assign_message(
        assign_message_task_shutdown_receiver,
        messages_to_send_receiver,
        Arc::clone(&senders_of_write_tasks),
    ));

    // start write_task broker
    let shutdown_handles_of_write_tasks: Arc<Mutex<HashMap<SocketAddr, Sender<()>>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let write_task_broker_task = task::spawn(write_task_broker::write_task_broker(
        write_task_receiver,
        Arc::clone(&senders_of_write_tasks),
        Arc::clone(&shutdown_handles_of_write_tasks),
        connected_peers_sender,
    ));

    // remove_peer task
    let (remove_peer_shutdown_sender, remove_peer_shutdown_receiver) = mpsc::unbounded();
    let remove_peer_task = task::spawn(remove_peer::remove_peer(
        remove_peer_shutdown_receiver,
        peers_to_remove_receiver,
        Arc::clone(&shutdown_handles_of_read_tasks),
        Arc::clone(&shutdown_handles_of_write_tasks),
        Arc::clone(&senders_of_write_tasks),
    ));

    // graceful shutdown
    task::spawn(graceful_shutdown::graceful_shutdown(
        graceful_shutdown_receiver,
        bind_task_shutdown_sender,
        add_peer_task_shutdown_sender,
        assign_message_task_shutdown_sender,
        remove_peer_shutdown_sender,
    ));

    // listen for incoming connections
    let mut incoming = listener.incoming();

    loop {
        let stream_result = select! {

            stream_option = incoming.next().fuse() => match stream_option {

               Some(stream_result) => stream_result,

               // The stream of connections is infinite, i.e awaiting the next connection will never result in None
               // https://docs.rs/async-std/0.99.9/async_std/net/struct.TcpListener.html#method.incoming
               None => {
                    unreachable!();
               }

            },

            void = bind_task_shutdown_receiver.next().fuse() => match void {
                Some(()) => {
                    logger::info("Received shutdown signal.");
                    break
                }
                None => break,
            }

        };

        match stream_result {
            Ok(stream) => {
                logger::info("Client connected.");
                tcp_stream_sender.send(stream).await.unwrap();
            }

            Err(_error) => {
                logger::warn("Client cannot be accepted");
            }
        }
    }

    drop(tcp_stream_sender);

    add_peer_task.await;
    process_stream_task.await;
    read_task_broker_task.await;
    assign_message_task.await;
    write_task_broker_task.await;
    remove_peer_task.await;

    Ok(())
}

#[derive(Clone)]
pub struct TcpServerConfig {
    pub address: String,
}

#[derive(Clone)]
pub struct TcpClientConfig {
    pub address: String,
}
