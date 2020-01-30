use std::{
    sync::Arc,
    collections::HashMap,
};

use futures::{channel::mpsc, select, FutureExt, SinkExt, lock::Mutex};

use async_std::{
    net::SocketAddr,
    prelude::*,
};
use crate::message::MessageToSend;

type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;

pub async fn remove_peer(mut shutdown_receiver: Receiver<()>, mut peers_to_remove_receiver: Receiver<SocketAddr>, shutdown_handles_of_read_tasks: Arc<Mutex<HashMap<SocketAddr, Sender<()>>>>, shutdown_handles_of_write_tasks: Arc<Mutex<HashMap<SocketAddr, Sender<()>>>>, senders_of_write_tasks: Arc<Mutex<HashMap<SocketAddr, Sender<MessageToSend>>>>) {

    loop {

        let address = select! {

            address_option = peers_to_remove_receiver.next().fuse() => match address_option {
               Some(address) => address,
               None => break,
            },

            void = shutdown_receiver.next().fuse() => match void {
                Some(()) => break,
                None => break,
            }

        };

        let shutdown_handles_of_read_tasks: &mut HashMap<SocketAddr, Sender<()>> = &mut *shutdown_handles_of_read_tasks.lock().await;
        let shutdown_handles_of_write_tasks: &mut HashMap<SocketAddr, Sender<()>> = &mut *shutdown_handles_of_write_tasks.lock().await;

        match shutdown_handles_of_read_tasks.remove(&address) {

            Some(mut read_task_shutdown_sender) => {
                read_task_shutdown_sender.send(()).await.unwrap()
            },
            None => {
                eprintln!("can not shutdown read_task of {}", address);
            }

        }

        match shutdown_handles_of_write_tasks.remove(&address) {

            Some(mut write_task_shutdown_sender) => {
                write_task_shutdown_sender.send(()).await.unwrap()
            },
            None => {
                eprintln!("can not shutdown write_task of {}", address);
            }

        }

        // remove peer from message_assigner
        (&mut *senders_of_write_tasks.lock().await).remove(&address);

    }

}