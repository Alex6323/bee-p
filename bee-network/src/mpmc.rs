use std::{
    sync::Arc
};

use futures::{channel::mpsc, channel::mpsc::UnboundedSender, channel::mpsc::UnboundedReceiver, SinkExt};

use async_std::{
    prelude::*,
    task,
    sync::Mutex
};

pub async fn unbounded<T>() -> (Sender<T>, Receiver<T>) where T: 'static + std::marker::Send + std::marker::Sync + std::clone::Clone {

    let (incoming_msg_sender, incoming_msg_receiver) = mpsc::unbounded();

    let registered_senders: Arc<Mutex<Vec<UnboundedSender<T>>>> = Arc::new(Mutex::new(Vec::new()));

    task::spawn(receive(incoming_msg_receiver, registered_senders.clone()));

    (Sender::new(incoming_msg_sender), Receiver::new(registered_senders.clone()).await)

}

async fn receive<T>(mut incoming_messages_receiver: UnboundedReceiver<T>, registered_senders: Arc<Mutex<Vec<UnboundedSender<T>>>>) where T: std::clone::Clone {

    while let Some(event) = incoming_messages_receiver.next().await {
        let set = &*registered_senders.lock().await;
        for sender in set {
            let mut sender: &UnboundedSender<T> = sender;
            sender.send(event.clone()).await;
        }
    }

}

pub struct Sender<T> {
    sender: mpsc::UnboundedSender<T>
}

impl<T> Sender<T> {

    fn new(sender: UnboundedSender<T>) -> Self {
        Self {
            sender
        }
    }

    pub async fn send(&mut self, data: T) -> Result<(), mpsc::SendError> {
        self.sender.send(data).await
    }

    pub fn clone(&self) -> Self {
       Sender { sender:  self.sender.clone() }
    }

}

pub struct Receiver<T> {
    receiver: mpsc::UnboundedReceiver<T>,
    registered_senders: Arc<Mutex<Vec<UnboundedSender<T>>>>
}

impl<T> Receiver<T> {

    async fn new(registered_senders: Arc<Mutex<Vec<UnboundedSender<T>>>>) -> Self {
        let (sender, receiver) = mpsc::unbounded::<T>();
        let locked_vec = &mut  *registered_senders.lock().await;
        locked_vec.push(sender);
        Self {receiver, registered_senders: Arc::clone(&registered_senders)}
    }

    pub async fn next(&mut self) -> Option<T> {
        self.receiver.next().await
    }

    pub async fn clone(&mut self) -> Self {
        let (sender, receiver) = mpsc::unbounded::<T>();
        let locked_vec = &mut  self.registered_senders.lock().await;
        locked_vec.push(sender);
        Self {receiver, registered_senders: Arc::clone(&self.registered_senders)}
    }

}

