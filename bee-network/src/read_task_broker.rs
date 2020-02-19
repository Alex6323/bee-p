use std::{
    collections::HashMap,
    convert::TryFrom,
    io::{Error, ErrorKind},
    sync::Arc,
};

use futures::{channel::mpsc, lock::Mutex, select, FutureExt, SinkExt};

use async_std::{
    io::BufReader,
    net::{SocketAddr, TcpStream},
    prelude::*,
    task,
};

use crate::message::{Message, MessageReader, ReceivedMessage};

use std::ops::Deref;

type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;

pub async fn read_task_broker<R>(
    mut read_task_receiver: Receiver<Arc<TcpStream>>,
    received_messages_sender: Sender<ReceivedMessage<R::MessageType>>,
    shutdown_handles_of_read_tasks: Arc<Mutex<HashMap<SocketAddr, Sender<()>>>>,
) where
    R: MessageReader + 'static,
    <R as MessageReader>::MessageType:
        Deref<Target = Message<Error = R::Error>> + std::marker::Send,
{
    while let Some(stream) = read_task_receiver.next().await {
        match stream.peer_addr() {
            Ok(address) => {
                let (read_task_shutdown_sender, read_task_shutdown_receiver) = mpsc::unbounded();
                let shutdown_handles_of_read_tasks: &mut HashMap<SocketAddr, Sender<()>> =
                    &mut *shutdown_handles_of_read_tasks.lock().await;
                shutdown_handles_of_read_tasks.insert(address, read_task_shutdown_sender);
                spawn_and_log_error(read_task::<R>(
                    read_task_shutdown_receiver,
                    stream,
                    received_messages_sender.clone(),
                ));
            }

            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }
}

async fn read_task<R>(
    mut shutdown_task: Receiver<()>,
    stream: Arc<TcpStream>,
    mut received_messages: Sender<ReceivedMessage<R::MessageType>>,
) -> Result<(), Error>
where
    R: MessageReader,
{
    // let mut reader = BufReader::new(&*stream);

    loop {
        let mut reader = BufReader::new(&*stream);
        let message = R::read(reader)
            .await
            .map_err(|_| Error::new(ErrorKind::Other, "oh no!"))?;
        // // 1) Check message type
        // let mut message_type_buf = [0u8; 1];
        // select! {
        //     result = reader.read_exact(&mut message_type_buf).fuse() => result?,
        //     void = shutdown_task.next().fuse() => break
        // }
        // let message_type = u8::from_be_bytes(message_type_buf);
        //
        // // 2) Check message length
        // let mut message_length_buf = [0u8; 2];
        // select! {
        //     result = reader.read_exact(&mut message_length_buf).fuse() => result?,
        //     void = shutdown_task.next().fuse() => break
        // }
        // let message_length_as_usize = usize::try_from(u16::from_be_bytes(message_length_buf));
        //
        // if let Err(_) = message_length_as_usize {
        //     return Err(Error::new(
        //         ErrorKind::InvalidInput,
        //         "Can't convert message_length to usize",
        //     ));
        // }
        //
        // let message_length = message_length_as_usize.unwrap();

        // 3) Parse message based on type and length
        // match message_type {
        //     1 => {
        //         let mut test_message_buf = vec![0u8; message_length];
        //         select! {
        //             result = reader.read_exact(&mut test_message_buf).fuse() => result?,
        //             void = shutdown_task.next().fuse() => break
        //         }
        //
        received_messages
            .send(ReceivedMessage {
                from: stream.peer_addr()?,
                msg: message,
            })
            .await
            .unwrap();
        // }
        //
        //     _ => return Err(Error::new(ErrorKind::InvalidInput, "Invalid message type")),
        // }
    }

    Ok(())
}

fn spawn_and_log_error<F>(fut: F) -> task::JoinHandle<()>
where
    F: Future<Output = Result<(), Error>> + Send + 'static,
{
    task::spawn(async move {
        if let Err(e) = fut.await {
            eprintln!("{}", e)
        }
    })
}
