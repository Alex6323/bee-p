// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use err_derive::Error;

// #[derive(Debug, Error)]
// pub enum ActorError {
//     #[error(display = "Async IO error")]
//     AsyncIo(#[source] async_std::io::Error),

//     #[error(display = "Error sending message")]
//     SendingMessageFailed(#[source] futures::channel::mpsc::SendError),
// }

// pub type Result<T> = std::result::Result<T, ActorError>;

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
// Io(#[source] async_std::io::Error),
}

// pub type SendResult<T> = std::result::Result<T, SendError>;

#[derive(Debug, Error)]
pub enum RecvError {
    #[error(display = "Receiving Error")]
    Io(#[source] async_std::io::Error),
}

// pub type RecvResult<T> = std::result::Result<T, RecvError>;
