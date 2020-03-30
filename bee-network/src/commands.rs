use crate::{
    address::url::Url,
    endpoint::EndpointId,
};

use futures::channel::{
    mpsc,
    oneshot,
};

use std::fmt;

// TODO: do not expose `futures::Sender, futures::Receiver` directly to make sure
// we can version up independently
// TODO: we don't need this to be generic atm; just use `bool` for now
/// Receiver half of the notification channel.
pub type Responder<T> = oneshot::Sender<T>;
/// Sender half of the notification channel.
pub type Requester<T> = oneshot::Receiver<T>;

/// Creates a channel for returning success/failure notfication.
pub fn response_channel<T>() -> (Responder<T>, Requester<T>) {
    oneshot::channel::<T>()
}

/// `Command`s that can be sent to the network layer.
#[derive(Debug)]
pub enum Command {
    /// Adds an `Endpoint`.
    AddEndpoint {
        /// `Url` of the `Endpoint`.
        url: Url,

        /// Success responder.
        responder: Option<Responder<bool>>,
    },

    /// Removes an `Endpoint`.
    RemoveEndpoint {
        /// The id of the `Endpoint` to remove.
        epid: EndpointId,

        /// Success responder.
        responder: Option<Responder<bool>>,
    },

    /// Connects to an `Endpoint`.
    Connect {
        /// The id of the `Endpoint` to connect.
        epid: EndpointId,

        /// Sucess responder.
        responder: Option<Responder<bool>>,
    },

    /// Disconnects from an `Endpoint`.
    Disconnect {
        /// The id of the `Endpoint` to disconnect from.
        epid: EndpointId,

        /// Success responder.
        responder: Option<Responder<bool>>,
    },

    // TODO: rename to `SendMessage`
    /// Sends a message to a connected `Endpoint`.
    SendBytes {
        /// The id of the `Endpoint` to send the message to.
        epid: EndpointId,

        /// The raw bytes of the message.
        bytes: Vec<u8>,

        /// Success responder.
        responder: Option<Responder<bool>>,
    },

    // TODO: rename to `MulticastMessage`
    /// Sends a message to multiple connected `Endpoint`s.
    MulticastBytes {
        ///  The ids of `Endpoint`s to connect to.
        epids: Vec<EndpointId>,

        /// The raw bytes of the message.
        bytes: Vec<u8>,

        /// Success responder.
        responder: Option<Responder<bool>>,
    },

    // TODO: rename to `BroadcastMessage`
    /// Sends a message to all connected `Endpoint`s.
    BroadcastBytes {
        /// The raw bytes of the message.
        bytes: Vec<u8>,

        /// Success responder.
        responder: Option<Responder<bool>>,
    },
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::AddEndpoint { url, .. } => write!(f, "Command::AddEndpoint {{ {} }}", url),

            Command::RemoveEndpoint { epid, .. } => write!(f, "Command::RemoveEndpoint {{ {} }}", epid),

            Command::Connect { epid, .. } => write!(f, "Command::Connect {{ {} }}", epid),

            Command::Disconnect { epid, .. } => write!(f, "Command::Disconnect {{ {} }}", epid),

            Command::SendBytes { epid, .. } => write!(f, "Command::SendBytes {{ {} }}", epid),

            Command::MulticastBytes { epids, .. } => {
                write!(f, "Command::MulticastBytes {{ {} receivers }}", epids.len())
            }

            Command::BroadcastBytes { .. } => write!(f, "Command::BroadcastBytes"),
        }
    }
}

pub type CommandSender = mpsc::Sender<Command>;
pub type CommandReceiver = mpsc::Receiver<Command>;

// TODO: what's a good value here?
// TODO: put this into `constants.rs`
const COMMAND_CHANNEL_CAPACITY: usize = 1000;

pub(crate) fn command_channel() -> (CommandSender, CommandReceiver) {
    mpsc::channel(COMMAND_CHANNEL_CAPACITY)
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::{
        prelude::*,
        task::{
            block_on,
            spawn,
        },
    };
    use futures::sink::SinkExt;

    #[test]
    fn issue_fire_and_forget_command() {
        let (mut sender, mut receiver) = command_channel();
        let url = block_on(Url::from_str_with_port("tcp://localhost:15600")).unwrap();
        let mut received_command = false;

        spawn(async move {
            sender
                .send(Command::AddEndpoint { url, responder: None })
                .await
                .unwrap();
        });

        block_on(async move {
            while let Some(command) = receiver.next().await {
                match command {
                    Command::AddEndpoint { url, .. } => {
                        assert_eq!("tcp://127.0.0.1:15600", url.to_string(), "Unexpected URL");
                        received_command = true;
                    }
                    _ => assert!(false, "Wrong command received"),
                }
            }
            assert!(received_command, "Command was not received");
        });
    }

    #[test]
    fn issue_command_that_responds() {
        let (mut sender, mut receiver) = command_channel();
        let (responder, requester) = response_channel::<bool>();
        let url = block_on(Url::from_str_with_port("tcp://localhost:15600")).unwrap();
        let mut received_command = false;
        let mut received_response = false;

        // 1) spawn a task which sends a command
        spawn(async move {
            sender
                .send(Command::AddEndpoint {
                    url,
                    responder: Some(responder),
                })
                .await
                .unwrap();
        });

        // 2) spawn another task which receives the command
        spawn(async move {
            while let Some(command) = receiver.next().await {
                match command {
                    Command::AddEndpoint { url, responder } => {
                        assert_eq!("tcp://127.0.0.1:15600", url.to_string(), "Unexpected URL");
                        received_command = true;

                        if let Some(responder) = responder {
                            responder.send(true).unwrap();
                        }
                    }
                    _ => assert!(false, "Wrong command received"),
                }
            }
            assert!(received_command, "Command was not received");
        });

        // 3) wait for receiving the response
        block_on(async move {
            if let Ok(success) = requester.await {
                assert!(success);
                received_response = true;
            }
            assert!(received_response, "Response was not received");
        });
    }
}
