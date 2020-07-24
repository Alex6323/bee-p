use crate::message::Header;

use bee_network::Address;

use futures::stream::StreamExt;
use futures::{
    channel::{mpsc, oneshot},
    future, select, stream,
};
use log::debug;

enum PeerReadState {
    Header,
    Payload(Header),
}

pub(super) struct MessageHandler {
    events: EventHandler,
    state: PeerReadState,
    address: Address,
}

impl MessageHandler {
    pub(super) fn new(
        receiver_fused: stream::Fuse<mpsc::Receiver<Vec<u8>>>,
        shutdown_fused: future::Fuse<oneshot::Receiver<()>>,
        address: Address,
    ) -> Self {
        Self {
            events: EventHandler::new(receiver_fused, shutdown_fused),
            state: PeerReadState::Header,
            address,
        }
    }

    pub(super) async fn fetch_message<'a>(&'a mut self) -> Option<(Header, &'a [u8])> {
        loop {
            match &self.state {
                PeerReadState::Header => {
                    let bytes = self.events.fetch_bytes(3).await?;
                    debug!("[{}] Reading Header...", self.address);
                    let header = Header::from_bytes(bytes);
                    self.state = PeerReadState::Payload(header);
                }
                PeerReadState::Payload(header) => {
                    let header = header.clone();
                    let bytes = self.events.fetch_bytes(header.message_length as usize).await?;
                    self.state = PeerReadState::Header;
                    return Some((header, bytes));
                }
            }
        }
    }

    pub(super) fn consume(
        self,
    ) -> (
        stream::Fuse<mpsc::Receiver<Vec<u8>>>,
        future::Fuse<oneshot::Receiver<()>>,
    ) {
        (self.events.receiver_fused, self.events.shutdown_fused)
    }
}

struct EventHandler {
    receiver_fused: stream::Fuse<mpsc::Receiver<Vec<u8>>>,
    shutdown_fused: future::Fuse<oneshot::Receiver<()>>,
    buffer: Vec<u8>,
    offset: usize,
    closed: bool,
}

impl EventHandler {
    fn new(
        receiver_fused: stream::Fuse<mpsc::Receiver<Vec<u8>>>,
        shutdown_fused: future::Fuse<oneshot::Receiver<()>>,
    ) -> Self {
        Self {
            receiver_fused,
            shutdown_fused,
            buffer: vec![],
            offset: 0,
            closed: false,
        }
    }

    fn push_event(&mut self, mut bytes: Vec<u8>) {
        self.buffer = self.buffer.split_off(self.offset);
        self.offset = 0;

        if self.buffer.is_empty() {
            self.buffer = bytes;
        } else {
            self.buffer.append(&mut bytes);
        }
    }

    async fn fetch_bytes<'a>(&'a mut self, len: usize) -> Option<&'a [u8]> {
        if self.closed {
            return None;
        }

        while self.offset + len > self.buffer.len() {
            select! {
                event = self.receiver_fused.next() => {
                    if let Some(event) = event {
                        self.push_event(event);
                    }
                },
                _ = &mut self.shutdown_fused => {
                    self.closed = true;
                    return None;
                }
            }
        }

        let item = &self.buffer[self.offset..(self.offset + len)];
        self.offset += len;
        Some(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use futures::channel::{mpsc, oneshot};
    use futures::{future::FutureExt, stream::StreamExt};

    use async_std::task;

    use std::time::Duration;

    fn gen_events(event_len: usize, msg_size: usize, n_msg: usize) -> Vec<Vec<u8>> {
        let msg_len = ((msg_size - 3) as u16).to_be_bytes();

        let mut msgs = vec![0u8; msg_size * n_msg];

        for i in (0..n_msg).map(|i| i * msg_size + 1) {
            msgs[i] = msg_len[0];
            msgs[i + 1] = msg_len[1];
        }

        msgs.chunks(event_len).map(Vec::from).collect()
    }

    async fn test(event_size: usize, msg_size: usize, msg_count: usize) {
        let msg_len = msg_size - 3;

        let events = gen_events(event_size, msg_size, msg_count);

        let (sender_shutdown, receiver_shutdown) = oneshot::channel::<()>();
        let (mut sender, receiver) = mpsc::channel::<Vec<u8>>(9999);

        let mut msg_handler = MessageHandler::new(
            receiver.fuse(),
            receiver_shutdown.fuse(),
            Address::from_addr_str("127.0.0.1:8080").await.unwrap(),
        );

        let handle = task::spawn(async move {
            let expected_bytes = vec![0u8; msg_len];
            let expected_msg = (
                Header {
                    message_type: 0,
                    message_length: msg_len as u16,
                },
                expected_bytes.as_slice(),
            );

            let mut counter = 0;
            while let Some(msg) = msg_handler.fetch_message().await {
                assert_eq!(msg, expected_msg);
                counter += 1;
            }

            assert_eq!(msg_count, counter);

            msg_handler
        });

        for event in events {
            sender.try_send(event).unwrap();
        }

        task::sleep(Duration::from_secs(1)).await;

        sender_shutdown.send(()).unwrap();

        handle.await;
    }

    #[async_std::test]
    async fn one_byte_events() {
        test(1, 5, 10).await;
    }

    #[async_std::test]
    async fn one_message_per_event() {
        test(5, 5, 10).await;
    }

    #[async_std::test]
    async fn two_messages_per_event() {
        test(10, 5, 10).await;
    }

    #[async_std::test]
    async fn two_events_per_message() {
        test(5, 10, 10).await;
    }

    #[async_std::test]
    async fn misaligned_messages() {
        test(3, 5, 10).await;
    }

    #[async_std::test]
    async fn shutdown() {
        let event_size = 5;
        let msg_size = event_size;
        let msg_count = 10;

        let msg_len = msg_size - 3;

        let mut events = gen_events(event_size, msg_size, msg_count);
        let last_event = events.pop().unwrap();

        let (sender_shutdown, receiver_shutdown) = oneshot::channel::<()>();
        let (mut sender, receiver) = mpsc::channel::<Vec<u8>>(9999);

        let mut msg_handler = MessageHandler::new(
            receiver.fuse(),
            receiver_shutdown.fuse(),
            Address::from_addr_str("127.0.0.1:8080").await.unwrap(),
        );

        let handle = task::spawn(async move {
            let expected_bytes = vec![0u8; msg_len];
            let expected_msg = (
                Header {
                    message_type: 0,
                    message_length: msg_len as u16,
                },
                expected_bytes.as_slice(),
            );

            let mut counter = 0;
            while let Some(msg) = msg_handler.fetch_message().await {
                assert_eq!(msg, expected_msg);
                counter += 1;
            }
            assert_eq!(msg_count - 1, counter);

            msg_handler
        });

        for event in events {
            sender.try_send(event).unwrap();
        }

        task::sleep(Duration::from_secs(1)).await;

        sender_shutdown.send(()).unwrap();

        task::sleep(Duration::from_secs(1)).await;

        sender.try_send(last_event).unwrap();

        handle.await;
    }
}
