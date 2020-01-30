use futures::{channel::mpsc, select, FutureExt, SinkExt};

use async_std::{
    net::TcpStream,
    prelude::*
};

type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;

pub async fn add_peer(mut shutdown_receiver: Receiver<()>, mut client_config_receiver: Receiver<TcpClientConfig>, mut tcp_stream_sender: Sender<TcpStream>) {

    loop {

        let client_config = select! {

            client_config = client_config_receiver.next().fuse() => match client_config {
               Some(peer_config) => peer_config,
               None => break,
            },

            void = shutdown_receiver.next().fuse() => match void {
                Some(()) => break,
                None => break,
            }

        };

        match TcpStream::connect(client_config.address.clone()).await {

            Ok(stream) => {
                tcp_stream_sender.send(stream).await.unwrap();
            },

            Err(_error) => {
                eprintln!("can not accept client {}", client_config.address.clone());
            }

        }

    }

}

#[derive(Clone)]
pub struct TcpClientConfig {
    pub address: String,
}