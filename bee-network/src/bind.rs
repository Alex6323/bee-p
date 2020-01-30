use futures::{channel::mpsc, select, FutureExt, SinkExt};

use async_std::{
    net::{TcpListener, TcpStream},
    prelude::*,
};

type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;

pub async fn bind(mut shutdown_receiver: Receiver<()>, config: TcpServerConfig, mut tcp_stream_sender: Sender<TcpStream>) {

    // Bind server to provided address.
    let listener = match TcpListener::bind(config.address).await {
        Ok(listener) => listener,
        Err(_) => return println!("Can not bind server")
    };

    // Listen for incoming connections.
    let mut incoming = listener.incoming();

    loop {

        let stream_result = select! {

            stream_option = incoming.next().fuse() => match stream_option {

               Some(stream_result) => stream_result,

               // The stream of connections is infinite, i.e awaiting the next connection will never result in None
               // https://docs.rs/async-std/0.99.9/async_std/net/struct.TcpListener.html#method.incoming
               None => {
                    unreachable!();
               }

            },

            void = shutdown_receiver.next().fuse() => match void {
                Some(()) => break,
                None => break,
            }

        };

        match stream_result {

            Ok(stream) => {
                tcp_stream_sender.send(stream).await.unwrap();
            },

            Err(_error) => {
                eprintln!("can not accept client");
            }

        }

    }

}

#[derive(Clone)]
pub struct TcpServerConfig {
    pub address: String,
}