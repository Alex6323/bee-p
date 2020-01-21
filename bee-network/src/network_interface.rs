use crate::tcp_server::TcpServerConfig;

use crate::tcp_server::MessageToSend;
use crate::tcp_server::ReceivedMessage;
use crate::tcp_server::TcpClientConfig;
use crate::tcp_server;

use crate::mpmc;
use futures::channel::mpsc;

use async_std::task;
use async_std::net::SocketAddr;

pub async fn new(server_config: TcpServerConfig) -> NetworkAccessHandles {

    let (messages_to_send_sender, messages_to_send_receiver) = mpmc::unbounded().await;
    let (received_messages_sender, received_messages_receiver) = mpmc::unbounded().await;
    let (peers_to_add_sender, peers_to_add_receiver) = mpsc::unbounded();
    let (shutdown_sender, shutdown_receiver) = mpmc::unbounded().await;
    let (peers_sender, peers_receiver) = mpmc::unbounded().await;

    task::spawn(tcp_server::start(server_config, messages_to_send_receiver, received_messages_sender, peers_to_add_receiver, peers_sender, shutdown_receiver, ));

    NetworkAccessHandles {
        messages_to_send_sender,
        received_messages_receiver,
        peers_to_add_sender,
        shutdown_sender,
        peers_receiver
    }

}

pub struct NetworkAccessHandles {
    pub messages_to_send_sender: mpmc::Sender<MessageToSend>,
    pub received_messages_receiver: mpmc::Receiver<ReceivedMessage>,
    pub peers_to_add_sender: mpsc::UnboundedSender<TcpClientConfig>,
    pub shutdown_sender: mpmc::Sender<()>,
    pub peers_receiver: mpmc::Receiver<SocketAddr>
}

