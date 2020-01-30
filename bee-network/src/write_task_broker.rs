use std::{
    sync::Arc,
    io::{Error, ErrorKind},
    collections::HashMap,
    convert::TryInto,
};

use futures::{channel::mpsc, select, FutureExt, SinkExt, lock::Mutex};

use async_std::{
    net::{TcpStream, SocketAddr},
    prelude::*,
    task
};

use crate::message::Message;
use crate::message::MessageType;
use crate::message::MessageToSend;

type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;

pub async fn write_task_broker(mut write_task_receiver: Receiver<Arc<TcpStream>>, senders_of_write_tasks: Arc<Mutex<HashMap<SocketAddr, Sender<MessageToSend>>>>, shutdown_handles_of_write_tasks: Arc<Mutex<HashMap<SocketAddr, Sender<()>>>>, mut connected_peers_sender: Sender<SocketAddr>) {

    while let Some(stream) = write_task_receiver.next().await {

        match stream.peer_addr() {

            Ok(address) => {

                // register shutdown_sender of individual write_task
                let (write_task_shutdown_sender, write_task_shutdown_receiver) = mpsc::unbounded();
                let shutdown_handles_of_write_tasks: &mut HashMap<SocketAddr, Sender<()>> = &mut *shutdown_handles_of_write_tasks.lock().await;
                shutdown_handles_of_write_tasks.insert(address.clone(), write_task_shutdown_sender);

                // register message_sender of individual write_task
                let (write_task_message_sender, write_task_message_receiver) = mpsc::unbounded();
                let senders_of_write_tasks: &mut HashMap<SocketAddr, Sender<MessageToSend>> = &mut *senders_of_write_tasks.lock().await;
                senders_of_write_tasks.insert(address.clone(), write_task_message_sender);

                connected_peers_sender.send(address).await.unwrap();

                spawn_and_log_error(write_task(write_task_shutdown_receiver, stream, write_task_message_receiver));

            },

            Err(e) => {
                eprintln!("{}", e);
            }

        }

    }

}

async fn write_task(mut shutdown_task: Receiver<()>, stream: Arc<TcpStream>, mut message_receiver: Receiver<MessageToSend>) -> Result<(), Error> {

    let mut stream = &*stream;

    loop {

        let message = select! {

            message = message_receiver.next().fuse() => match message {
               Some(message) => message,
               None => break,
            },

            void = shutdown_task.next().fuse() => break

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

fn spawn_and_log_error<F>(fut: F) -> task::JoinHandle<()> where F: Future<Output = Result<(), Error>> + Send + 'static {
    task::spawn(async move {
        if let Err(e) = fut.await {
            eprintln!("{}", e)
        }
    })
}