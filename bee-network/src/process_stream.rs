use std::{
    sync::Arc
};

use futures::{channel::mpsc, SinkExt};

use async_std::{
    net::TcpStream,
    prelude::*
};

type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;

pub async fn process_stream(mut tcp_stream_receiver: Receiver<TcpStream>, mut read_task_sender: Sender<Arc<TcpStream>>, mut write_task_sender: Sender<Arc<TcpStream>>) {

    while let Some(stream) = tcp_stream_receiver.next().await {

        let stream = Arc::new(stream);

        read_task_sender.send(Arc::clone(&stream)).await.unwrap();
        write_task_sender.send(Arc::clone(&stream)).await.unwrap();

    }

}