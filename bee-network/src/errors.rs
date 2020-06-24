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

// TODO: Try to get rid of `XXXError` pattern; use `xxx::Error` instead
// TODO: Use thiserror crate instead because it's a bit less verbose
use err_derive::Error;

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

// TODO: remove alias bc it's considered a nono as of now
pub type ConnectionResult<T> = std::result::Result<T, ConnectionError>;

// TODO: fix this
#[derive(Debug, Error)]
pub enum SendError {
    //#[error(display = "Sending Error")]
// Io(#[source] async_std::io::Error),
}

#[derive(Debug, Error)]
pub enum RecvError {
    #[error(display = "Receiving Error")]
    Io(#[source] async_std::io::Error),
}
