use crate::commands::{
    Command,
    CommandSender,
};

use err_derive::Error;
use futures::sink::SinkExt;

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error(display = "error sending command")]
    CommandSendFailure(#[source] futures::channel::mpsc::SendError),
}

pub type NetworkResult = std::result::Result<(), NetworkError>;

#[derive(Clone, Debug)]
pub struct Network {
    inner: CommandSender,
}

impl Network {
    pub fn new(cmd_sender: CommandSender) -> Self {
        Self { inner: cmd_sender }
    }

    pub async fn send(&mut self, command: Command) -> NetworkResult {
        Ok(self.inner.send(command).await?)
    }
}
