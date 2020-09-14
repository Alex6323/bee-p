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

use crate::{endpoint::EndpointId, events::EventSender};

use super::{
    connection::{Connection, Origin},
    Error,
};

use log::*;
use tokio::net::TcpStream;

use std::net::SocketAddr;

pub(crate) async fn connect_endpoint(
    epid: EndpointId,
    socket_address: SocketAddr,
    internal_event_sender: EventSender,
) -> Result<(), Error> {
    debug!("Trying to connect to {}...", epid);

    match TcpStream::connect(socket_address).await {
        Ok(stream) => {
            let connection = match Connection::new(stream, Origin::Outbound) {
                Ok(conn) => conn,
                Err(e) => {
                    warn!["Error creating TCP connection: {:?}.", e];

                    return Err(Error::ConnectionAttemptFailed);
                }
            };

            debug!(
                "Sucessfully established connection to {} ({}).",
                connection.peer_address, connection.origin,
            );

            super::spawn_connection_workers(connection, internal_event_sender).await?;

            Ok(())
        }
        Err(e) => {
            warn!("Connecting to {} failed: {:?}.", epid, e);

            Err(Error::ConnectionAttemptFailed)
        }
    }
}
