use futures::{channel::mpsc, SinkExt};

use async_std::prelude::*;

type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;

pub async fn graceful_shutdown(

    mut graceful_shutdown_receiver: Receiver<()>,
    mut bind_task_shutdown_sender: Sender<()>,
    mut add_peer_task_shutdown_sender: Sender<()>,
    mut assign_message_task_shutdown_sender: Sender<()>,
    mut remove_peer_task_shutdown_sender: Sender<()>,

    ) {

    while let Some(()) = graceful_shutdown_receiver.next().await {
        bind_task_shutdown_sender.send(()).await.unwrap();
        add_peer_task_shutdown_sender.send(()).await.unwrap();
        assign_message_task_shutdown_sender.send(()).await.unwrap();
        remove_peer_task_shutdown_sender.send(()).await.unwrap();
    }

}