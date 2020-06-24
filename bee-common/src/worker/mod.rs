use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    /// A wrapper for an async task error.
    #[error("An asynchronous operation failed.")]
    AsynchronousOperationFailed(#[from] async_std::io::Error),

    /// An error that occurs, when sending a message over an `mpsc` channel failed.
    #[error("Sending a message to a task failed.")]
    SendingMessageFailed(#[from] futures::channel::mpsc::SendError),
}
