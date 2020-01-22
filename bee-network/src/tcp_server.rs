use std::{
    sync::Arc,
    io::{Error, ErrorKind},
    collections::HashSet,
    convert::TryFrom,
    convert::TryInto,
    net::Shutdown
};

use futures::{channel::mpsc, select, FutureExt};

use async_std::{
    io::BufReader,
    net::{TcpListener, TcpStream, SocketAddr},
    prelude::*,
    task
};

use crate::message::{Message, MessageType};
use crate::mpmc;

type Receiver<T> = mpsc::UnboundedReceiver<T>;

pub async fn start(config: TcpServerConfig, mut messages_to_send: mpmc::Receiver<MessageToSend>, received_messages: mpmc::Sender<ReceivedMessage>, peers_to_add: Receiver<TcpClientConfig>, mut connected_peers: mpmc::Sender<SocketAddr>, mut shutdown_server: mpmc::Receiver<()>) -> Result<(), Error> {

    let listener = TcpListener::bind(config.address).await?;

    task::spawn(add_peer_task(peers_to_add, messages_to_send.clone().await, received_messages.clone(), connected_peers.clone(), shutdown_server.clone().await));

    let mut incoming = listener.incoming();

    loop {

        select! {

            stream = incoming.next().fuse() => match stream {

               Some(result) => {

                    match result {

                        Ok(stream) => {

                            let stream = Arc::new(stream);

                            let received_messages_clone = received_messages.clone();
                            let messages_to_send_clone = messages_to_send.clone().await;
                            let shutdown_server_clone = shutdown_server.clone().await;

                            match stream.peer_addr() {
                                Ok(address) => {
                                    connected_peers.send(address).await.unwrap()
                                },
                                Err(e) => {
                                    eprintln!("couldn't get peer address");
                                    continue;
                                }
                            }

                            spawn(read_task(Arc::clone(&stream), received_messages_clone));
                            spawn(write_task(Arc::clone(&stream), messages_to_send_clone, shutdown_server_clone));

                        },

                        Err(e) => {
                           eprintln!("couldn't accept client");
                            continue;
                        }

                    }

               },

               None => {
                    eprintln!("couldn't accept client");
                    break;
               }

            },

            void = shutdown_server.next().fuse() => match void {
                Some(()) => break,
                None => break,
            }

        };

    }

    Ok(())

}

async fn add_peer_task(mut peer_config_receiver: Receiver<TcpClientConfig>, mut messages_to_send: mpmc::Receiver<MessageToSend>, received_messages: mpmc::Sender<ReceivedMessage>,  mut connected_peers: mpmc::Sender<SocketAddr>, mut shutdown_server: mpmc::Receiver<()>) {

    loop {

        let peer_config = select! {

            peer_config = peer_config_receiver.next().fuse() => match peer_config {
               Some(peer_config) => peer_config,
               None => break,
            },
            void = shutdown_server.next().fuse() => match void {
                Some(()) => break,
                None => break,
            }

        };

        let peer_config: TcpClientConfig = peer_config;

        match TcpStream::connect(peer_config.address).await {

            Ok(stream) => {

                let stream = Arc::new(stream);

                let received_messages_clone = received_messages.clone();
                let messages_to_send_clone = messages_to_send.clone().await;
                let shutdown_server_clone = shutdown_server.clone().await;

                match stream.peer_addr() {
                    Ok(address) => {
                        connected_peers.send(address).await.unwrap()
                    },
                    Err(e) => {
                        eprintln!("couldn't get peer address");
                        continue;
                    }
                }

                spawn(read_task(Arc::clone(&stream), received_messages_clone));
                spawn(write_task(Arc::clone(&stream), messages_to_send_clone, shutdown_server_clone));

            },

            Err(e) => {
                eprintln!("couldn't accept client");
                continue;
            }

        }

    }

}

async fn write_task(stream: Arc<TcpStream>, mut message_receiver: mpmc::Receiver<MessageToSend>, mut shutdown_server: mpmc::Receiver<()>) -> Result<(), Error> {

    let mut stream = &*stream;

    loop {

        let message = select! {

            message = message_receiver.next().fuse() => match message {
               Some(message) => message,
               None => break,
            },
            void = shutdown_server.next().fuse() => match void {
                Some(()) => {stream.shutdown(Shutdown::Both)?; break; },
                None => break,
            }

        };

        let message: MessageToSend = message;

        if !message.to.is_empty() && !message.to.contains(&stream.peer_addr()?) {
            continue;
        }

        match message.msg {

            MessageType::Test(msg) => {

                let message_type = [1u8;1];
                let message_length;

                let message_length_result: Result<u16, std::num::TryFromIntError> = msg.bytes().len().try_into();

                if let Err(_) = message_length_result {
                    return Err(Error::new(ErrorKind::InvalidInput, "Message is too big"));
                }

                message_length = message_length_result.unwrap();

                stream.write_all(&message_type).await?;
                stream.write_all(&message_length.to_be_bytes()).await?;
                stream.write_all(msg.bytes()).await?;

            }

        }

    }

    Ok(())

}

async fn read_task(stream: Arc<TcpStream>, mut received_messages: mpmc::Sender<ReceivedMessage>) -> Result<(), Error> {

    let mut reader = BufReader::new(&*stream);

    loop {

        // 1) Check message type

        let mut message_type_buf = [0u8; 1];
        reader.read_exact(&mut message_type_buf).await?;
        let message_type = u8::from_be_bytes(message_type_buf);

        // 2) Check message length

        let mut message_length_buf = [0u8; 2];
        reader.read_exact(&mut message_length_buf).await?;
        let message_length_as_usize = usize::try_from(u16::from_be_bytes(message_length_buf));

        if let Err(_) = message_length_as_usize {
            return Err(Error::new(ErrorKind::InvalidInput, "Can't convert message_length to usize"));
        }

        let message_length = message_length_as_usize.unwrap();

        // 3) Parse message based on type and length
        match message_type {

            1 => {

                let mut test_message_buf = vec![0u8; message_length];
                reader.read_exact(&mut test_message_buf).await?;

                received_messages.send(ReceivedMessage{from: stream.peer_addr()?, msg: MessageType::Test(Message::new(test_message_buf)? ) }).await.unwrap();


            },

            _ => return Err(Error::new(ErrorKind::InvalidInput, "Invalid message type"))

        }


    }

}

#[derive(Clone)]
pub struct TcpClientConfig {
    pub address: String,
}

#[derive(Clone)]
pub struct TcpServerConfig {
    pub address: String,
}

#[derive(Clone)]
pub struct MessageToSend {
    pub to: HashSet<SocketAddr>,
    pub msg: MessageType
}

#[derive(Clone)]
pub struct ReceivedMessage {
    pub from: SocketAddr,
    pub msg: MessageType
}

fn spawn<F>(fut: F) -> task::JoinHandle<()> where F: Future<Output = Result<(), Error>> + Send + 'static {
    task::spawn(async move {
        if let Err(e) = fut.await {
            eprintln!("{}", e)
        }
    })
}