use std::{collections::HashMap, sync::Arc};

use futures::{channel::mpsc, lock::Mutex, select, FutureExt, SinkExt};

use async_std::{net::SocketAddr, prelude::*};

use crate::message::MessageToSend;

type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;

pub async fn assign_message<M>(
    mut shutdown_receiver: Receiver<()>,
    mut messages_to_send_receiver: Receiver<MessageToSend<M>>,
    senders_of_write_tasks: Arc<Mutex<HashMap<SocketAddr, Sender<MessageToSend<M>>>>>,
) where
    M: Clone,
{
    loop {
        let message = select! {

            message_option = messages_to_send_receiver.next().fuse() => match message_option {
               Some(message) => message,
               None => break
            },

            void = shutdown_receiver.next().fuse() => match void {
                Some(()) => break,
                None => break,
            }

        };

        let message: MessageToSend<M> = message;
        let map = &*senders_of_write_tasks.lock().await;

        if message.to.is_empty() {
            for (_key, mut value) in map {
                value.send(message.clone()).await.unwrap();
            }
        } else {
            for peer in &message.to {
                let message_sender = map.get(&peer);

                if let Some(mut sender) = message_sender {
                    sender.send(message.clone()).await.unwrap();
                } else {
                    eprintln!("peer with address {} not found", peer);
                }
            }
        }
    }
}
