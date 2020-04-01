pub mod connection;
pub mod worker;

use connection::TcpConnection;

use crate::{
    address::{
        url::Protocol,
        Address,
    },
    constants::MAX_BUFFER_SIZE,
    endpoint::{
        origin::Origin,
        outbox::{
            bytes_channel,
            BytesReceiver,
        },
        Endpoint,
        EndpointId as EpId,
    },
    errors::{
        ConnectionError,
        ConnectionResult,
    },
    events::{
        Event,
        EventPublisher as Notifier,
    },
};

use async_std::{
    net::TcpStream,
    sync::Arc,
    task::spawn,
};
use futures::{
    channel::oneshot,
    prelude::*,
    select,
};
use log::*;

/// Tries to connect to an endpoint.
pub(crate) async fn try_connect(epid: &EpId, addr: &Address, notifier: Notifier) -> ConnectionResult<()> {
    debug!("[TCP  ] Trying to connect to {}...", epid);

    match TcpStream::connect(**addr).await {
        Ok(stream) => {
            let conn = match TcpConnection::new(stream, Origin::Outbound) {
                Ok(conn) => conn,
                Err(e) => {
                    error!["TCP  ] Error creating TCP connection (Stream immediatedly aborted?)."];
                    error!["TCP  ] Error was: {:?}.", e];
                    return Err(ConnectionError::ConnectionAttemptFailed);
                }
            };

            debug!(
                "[TCP  ] Sucessfully established connection to {} ({}).",
                conn.remote_addr,
                Origin::Outbound
            );

            Ok(spawn_connection_workers(conn, notifier).await?)
        }
        Err(e) => {
            warn!("[TCP  ] Connection attempt failed (Endpoint offline?).");
            warn!("[TCP  ] Error was: {:?}.", e);
            Err(ConnectionError::ConnectionAttemptFailed)
        }
    }
}

pub(crate) async fn spawn_connection_workers(conn: TcpConnection, mut notifier: Notifier) -> ConnectionResult<()> {
    debug!("[TCP  ] Spawning TCP connection workers...");

    let addr: Address = conn.remote_addr.into();
    let proto = Protocol::Tcp;
    let origin = conn.origin;

    let ep = Endpoint::new(addr, proto);

    let (sender, receiver) = bytes_channel();
    let (shutdown_sender, shutdown_receiver) = oneshot::channel::<()>();

    spawn(writer(ep.id, conn.stream.clone(), receiver, shutdown_sender));
    spawn(reader(ep.id, conn.stream.clone(), notifier.clone(), shutdown_receiver));

    Ok(notifier.send(Event::NewConnection { ep, origin, sender }).await?)
}

async fn writer(epid: EpId, stream: Arc<TcpStream>, bytes_rx: BytesReceiver, sd: oneshot::Sender<()>) {
    debug!("[TCP  ] Starting connection writer task for {}...", epid);

    let mut stream = &*stream;
    let mut bytes_rx = bytes_rx.fuse();

    loop {
        select! {
            bytes_out = bytes_rx.next() => {
                if let Some(bytes_out) = bytes_out {

                    match stream.write_all(&*bytes_out).await {
                        Ok(_) => {
                            // NOTE: if we should need it, we can raise [`Event::BytesSent`] here.
                        },
                        Err(e) => {
                            error!("[TCP  ] Sending bytes failed.");
                            error!("[TCP  ] Error was: {:?}.", e);
                        }
                    }
                } else {
                    // NOTE: If the bytes sender gets dropped (which happens when the connection pool
                    // is dropped, we break out of the loop)
                    break;
                }
            }
        }
    }

    if sd.send(()).is_err() {
        trace!("[TCP  ] Reader task shut down before writer task.");
    }

    debug!("[TCP  ] Connection writer event loop for {} stopped.", epid);
}

async fn reader(epid: EpId, stream: Arc<TcpStream>, mut notifier: Notifier, mut sd: oneshot::Receiver<()>) {
    debug!("[TCP  ] Starting connection reader event loop for {}...", epid);

    let mut stream = &*stream;
    let mut buffer = vec![0; MAX_BUFFER_SIZE];
    let shutdown = &mut sd;

    loop {
        select! {
            num_read = stream.read(&mut buffer).fuse() => {
                match num_read {
                    Ok(num_read) => {
                        if num_read == 0 {
                            trace!("[TCP  ] Received EOF (0 byte message).");

                            if notifier.send(Event::LostConnection { epid }).await.is_err() {
                                warn!("[TCP  ] Failed to send 'LostConnection' notification.");
                            }

                            // NOTE: local reader shut down first (we were disconnected)
                            break;
                        } else {
                            let mut bytes = vec![0u8; num_read];
                            bytes.copy_from_slice(&buffer[0..num_read]);

                            if notifier.send(Event::MessageReceived { epid, bytes }).await.is_err() {
                                warn!("[TCP  ] Failed to send 'MessageReceived' notification.");
                            }
                        }
                    },
                    Err(e) => {
                        error!("[TCP  ] Receiveing bytes failed.");
                        error!("[TCP  ] Error was: {:?}.", e);
                    }
                }
            },
            shutdown = shutdown.fuse() => {
                // NOTE: local writer shut down first (we disconnected)
                break;
            }
        }
    }
    debug!("[TCP  ] Connection reader event loop for {} stopped.", epid);
}
