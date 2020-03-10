use crate::address::url::Url;
use crate::connection::ConnectionId;

use futures::channel::{mpsc, oneshot};

use std::fmt;

pub type Responder<T> = oneshot::Sender<T>;
pub type Requester<T> = oneshot::Receiver<T>;

pub fn response_channel<T>() -> (Responder<T>, Requester<T>) {
    oneshot::channel::<T>()
}

#[derive(Debug)]
pub enum Command {

    AddEndpoint {
        url: Url,
        responder: Option<Responder<Option<ConnectionId>>>,
    },

    RemoveEndpoint {
        conn: ConnectionId,
        responder: Option<Responder<bool>>,
    },

    Connect {
        conn: ConnectionId,
        attempts: Option<usize>,
        responder: Option<Responder<bool>>,
    },

    Disconnect {
        conn: ConnectionId,
        responder: Option<Responder<bool>>,
    },

    UnicastBytes {
        conn: ConnectionId,
        bytes: Vec<u8>,
    },

    MulticastBytes {
        conns: Vec<ConnectionId>,
        bytes: Vec<u8>,
    },

    BroadcastBytes {
        bytes: Vec<u8>,
    },

    Shutdown {
        responder: Option<Responder<bool>>,
    },
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::AddEndpoint { url, .. } =>
                write!(f, "Command::AddEndpoint {{ url = {:?} }} ", url),

            Command::RemoveEndpoint { conn, .. } =>
                write!(f, "Command::RemoveEndpoint {{ conn = {:?} }}", conn),

            Command::Connect { conn, attempts, .. } =>
                write!(f, "Command::Connect {{ conn = {:?}, attempts = {:?} }}", conn, attempts),

            Command::Disconnect { conn, .. } =>
                write!(f, "Command::Disconnect {{ conn = {:?} }}", conn),

            Command::UnicastBytes { conn, .. } =>
                write!(f, "Command::UnicastBytes {{ conn = {:?} }}", conn),

            Command::MulticastBytes { conns, .. } =>
                write!(f, "Command::MulticastBytes {{ num_conns = {} }}", conns.len()),

            Command::BroadcastBytes { .. } =>
                write!(f, "Command::BroadcastBytes"),

            Command::Shutdown { .. }=>
                write!(f, "Command::Shutdown"),
        }
    }
}

pub type CommandSender = mpsc::Sender<Command>;
pub type CommandReceiver = mpsc::Receiver<Command>;

// TODO: what's a good value here?
const COMMAND_CHANNEL_CAPACITY: usize = 1000;

pub(crate) fn command_channel() -> (CommandSender, CommandReceiver) {
    mpsc::channel(COMMAND_CHANNEL_CAPACITY)
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::task::{block_on, spawn};
    use futures::sink::SinkExt;

    #[test]
    fn issue_fire_and_forget_command() {
        let (mut sender, mut receiver) = command_channel();
        let url = block_on(Url::from_str_with_port("tcp://localhost:15600")).unwrap();
        let mut received_command = false;

        spawn(async move {
            sender.send(
                Command::AddEndpoint {
                    url,
                    responder: None,
                }).await.unwrap();
        });

        block_on(async move {
            while let Some(command) = receiver.next().await {
                match command {
                    Command::AddEndpoint { url, .. } => {

                        assert_eq!("tcp://127.0.0.1:15600", url.to_string(), "Unexpected URL");
                        received_command = true;
                    },
                    _ => assert!(false, "Wrong command received"),
                }
            }
            assert!(received_command, "Command was not received");
        });
    }

    #[test]
    fn issue_command_that_responds() {
        let (mut sender, mut receiver) = command_channel();
        let (responder, requester) = response_channel::<Option<ConnectionId>>();
        let url = block_on(Url::from_str_with_port("tcp://localhost:15600")).unwrap();
        let mut received_command = false;
        let mut received_response = false;

        // 1) spawn a task which sends a command
        spawn(async move {
            sender.send(
                Command::AddEndpoint {
                    url,
                    responder: Some(responder),
                }).await.unwrap();
        });

        // 2) spawn another task which receives the command
        spawn(async move {
            while let Some(command) = receiver.next().await {
                match command {
                    Command::AddEndpoint { url, responder } => {

                        assert_eq!("tcp://127.0.0.1:15600", url.to_string(), "Unexpected URL");
                        received_command = true;

                        if let Some(responder) = responder {
                            responder.send(Some(ConnectionId::from(url))).unwrap();
                        }
                    },
                    _ => assert!(false, "Wrong command received"),
                }
            }
            assert!(received_command, "Command was not received");
        });

        // 3) wait for receiving the response
        block_on(async move {
            if let Ok(conn_id) = requester.await {
                let conn_id = conn_id.unwrap();

                assert_eq!("127.0.0.1:15600", conn_id.to_string(), "Unexpected ID");
                received_response = true;
            }
            assert!(received_response, "Response was not received");
        });
    }
}