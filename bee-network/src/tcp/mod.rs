pub mod actor;
pub mod connection;

use connection::{
    ConnectionType,
    TcpConnection,
};

use crate::address::{
    url::Protocol,
    Address,
};
use crate::connection::{
    bytes_channel,
    BytesReceiver,
};
use crate::constants::MAX_BUFFER_SIZE;
use crate::endpoint::EndpointId as EpId;
use crate::errors::{
    ConnectionError,
    ConnectionSuccess as S,
};
use crate::events::{
    Event,
    EventPublisher as Notifier,
};

use async_std::net::TcpStream;
use async_std::sync::Arc;
use async_std::task::spawn;
use futures::channel::oneshot;
use futures::prelude::*;
use futures::select;
use log::*;

/// Tries to connect to an endpoint.
pub(crate) async fn try_connect(epid: &EpId, addr: &Address, notifier: Notifier) -> S {
    debug!("[TCP  ] Trying to connect to {:?} ...", epid);

    match TcpStream::connect(**addr).await {
        Ok(stream) => {
            let conn = match TcpConnection::new(stream, ConnectionType::Initiated) {
                Ok(conn) => conn,
                Err(e) => {
                    error!["TCP  ] Error creating TCP connection (Stream immediatedly aborted?)."];
                    error!["TCP  ] Error was: {:?}.", e];
                    return Err(ConnectionError::ConnectionAttemptFailed);
                }
            };

            debug!(
                "TCP  ] Sucessfully connected to endpoint {:?} ({:?}).",
                conn.epid,
                ConnectionType::Initiated
            );

            Ok(spawn_connection_workers(conn, notifier).await?)
        }
        Err(e) => {
            error!("[TCP  ] Connection attempt failed (Endpoint offline?).");
            error!("[TCP  ] Error was: {:?}.", e);
            Err(ConnectionError::ConnectionAttemptFailed)
        }
    }
}

pub(crate) async fn spawn_connection_workers(conn: TcpConnection, mut notifier: Notifier) -> S {
    debug!("[TCP  ] Spawning TCP connection workers...");

    let epid = conn.epid.clone();
    let addr: Address = conn.remote_addr.into();
    let prot = Protocol::Tcp;

    let (sender, receiver) = bytes_channel();
    let (shutdown_sender, shutdown_receiver) = oneshot::channel::<()>();

    spawn(writer(
        epid.clone(),
        conn.stream.clone(),
        receiver,
        notifier.clone(),
        shutdown_sender,
    ));

    spawn(reader(
        epid.clone(),
        addr.clone(),
        conn.stream.clone(),
        notifier.clone(),
        shutdown_receiver,
    ));

    Ok(notifier
        .send(Event::NewConnection {
            epid,
            addr,
            prot,
            sender,
        })
        .await?)
}

async fn writer(
    epid: EpId,
    stream: Arc<TcpStream>,
    mut bytes_rx: BytesReceiver,
    mut notifier: Notifier,
    sd: oneshot::Sender<()>,
) {
    debug!("[TCP  ] Starting connection writer task for {:?}...", epid);

    let mut stream = &*stream;

    loop {
        select! {
            bytes_out = bytes_rx.next().fuse() => {
                if let Some(bytes_out) = bytes_out {

                    match stream.write_all(&*bytes_out).await {
                        Ok(_) => {
                            // TODO: Is this event interesting at all, because if not, then
                            // we should not raise it and spare resources
                            match notifier.send(Event::BytesSent {
                                to: epid.clone(),
                                num: bytes_out.len(),
                            }).await {
                                Ok(_) => (),
                                Err(e) => {
                                    error!("[TCP  ] Failed notifying about bytes sent: {:?}.", e);
                                }
                            }
                        },
                        Err(e) => {
                            error!("[TCP  ] Sending bytes failed.");
                            error!("[TCP  ] Error was: {:?}", e);
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

    match sd.send(()) {
        Ok(_) => (),
        Err(_) => {
            error!("[TCP  ] Failed to send shutdown signal to reader task");
        }
    }

    debug!("[TCP  ] Connection writer event loop for {:?} stopped.", epid);
}

async fn reader(
    epid: EpId,
    addr: Address,
    stream: Arc<TcpStream>,
    mut notifier: Notifier,
    mut sd: oneshot::Receiver<()>,
) {
    debug!("[TCP  ] Starting connection reader event loop for {:?}...", epid);

    let mut stream = &*stream;
    let mut buffer = vec![0; MAX_BUFFER_SIZE];
    let shutdown = &mut sd;

    loop {
        select! {
            num_read = stream.read(&mut buffer).fuse() => {
                match num_read {
                    Ok(num_read) => {
                        if num_read == 0 {
                            warn!("[TCP  ] Received an empty message (0 bytes).");
                            continue;
                        } else {
                            let mut bytes = Vec::with_capacity(num_read);
                            bytes.copy_from_slice(&buffer[0..num_read]);

                            match notifier.send(Event::BytesReceived {
                                from: epid.clone(),
                                with_addr: addr.clone(),
                                bytes,
                            }).await {
                                Ok(_) => (),
                                Err(e) => {
                                    error!("[TCP  ] Failed notifying about bytes received: {:?}.", e);
                                }
                            }
                        }
                    },
                    Err(e) => {
                        error!("[TCP  ] Receiveing bytes failed.");
                        error!("[TCP  ] Error was: {:?}", e);
                    }
                }
            },
            shutdown = shutdown.fuse() => {
                break;
            }
        }
    }
    debug!("[TCP  ] Connection reader event loop for ({:?}) stopped.", epid);
}
