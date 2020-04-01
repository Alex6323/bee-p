use err_derive::Error;

#[derive(Debug, Error)]
pub enum ActorError {
    #[error(display = "Async IO error")]
    AsyncIo(#[source] async_std::io::Error),

    #[error(display = "Error sending message")]
    SendingMessageFailed(#[source] futures::channel::mpsc::SendError),
}

pub type Result<T> = std::result::Result<T, ActorError>;

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error(display = "Async IO error")]
    AsyncIo(#[source] async_std::io::Error),

    #[error(display = "Error sending bytes")]
    SendingBytesFailed(#[source] SendError),

    #[error(display = "Error receiving bytes")]
    ReceivingBytesFailed(#[source] RecvError),

    #[error(display = "Connection attempt failed")]
    ConnectionAttemptFailed,

    #[error(display = "Sending event failed")]
    SendingEventFailed(#[source] futures::channel::mpsc::SendError),
}

pub type ConnectionResult<T> = std::result::Result<T, ConnectionError>;

#[derive(Debug, Error)]
pub enum SendError {
    //#[error(display = "Sending Error")]
//Io(#[source] async_std::io::Error),
}

//pub type SendResult<T> = std::result::Result<T, SendError>;

#[derive(Debug, Error)]
pub enum RecvError {
    #[error(display = "Receiving Error")]
    Io(#[source] async_std::io::Error),
    //#[error(display = "Reading 0 bytes from stream")]
    //ZeroBytesReceived,
}

//pub type RecvResult<T> = std::result::Result<T, RecvError>;
