use crate::message::Header;

use bee_network::Address;

use futures::{
    channel::{mpsc, oneshot},
    future, select,
    stream::{self, StreamExt},
};

use log::debug;

type EventRecv = stream::Fuse<mpsc::Receiver<Vec<u8>>>;
type ShutdownRecv = future::Fuse<oneshot::Receiver<()>>;

/// The read state of the message handler.
///
/// This type is used by `MessageHandler` to decide what should be read next when handling an
/// event.
enum ReadState {
    /// `MessageHandler` should read a header.
    Header,
    /// `MessageHandler` should read a payload based on a header.
    Payload(Header),
}

/// A message handler
///
/// It takes care of processing events into messages that can be processed by the workers.
pub(super) struct MessageHandler {
    events: EventHandler,
    state: ReadState,
    /// The address of the peer. This field is only here for logging purposes.
    address: Address,
}

impl MessageHandler {
    /// Create a new message handler from an event receiver, a shutdown receiver and the peer's
    /// address.
    pub(super) fn new(receiver: EventRecv, shutdown: ShutdownRecv, address: Address) -> Self {
        Self {
            events: EventHandler::new(receiver, shutdown),
            // The handler should read a header first.
            state: ReadState::Header,
            address,
        }
    }
    /// Fetch the header and payload of a message.
    pub(super) async fn fetch_message<'a>(&'a mut self) -> Option<(Header, &'a [u8])> {
        // loop until we can return the header and payload
        loop {
            match &self.state {
                // Read a header.
                ReadState::Header => {
                    // We need 3 bytes to read a header.
                    let bytes = self.events.fetch_bytes(3).await?;
                    debug!("[{}] Reading Header...", self.address);
                    let header = Header::from_bytes(bytes);
                    // Now we are ready to read a payload.
                    self.state = ReadState::Payload(header);
                }
                // Read a payload.
                ReadState::Payload(header) => {
                    let header = header.clone();
                    // We read the quantity of bytes stated by the header.
                    let bytes = self.events.fetch_bytes(header.message_length as usize).await?;
                    // Now we are ready to read the next message's header.
                    self.state = ReadState::Header;
                    // We return the current message's header and payload.
                    return Some((header, bytes));
                }
            }
        }
    }
    // Helper method to recover the event and shutdown receivers.
    pub(super) fn consume(self) -> (EventRecv, ShutdownRecv) {
        (self.events.receiver, self.events.shutdown)
    }
}

// An event handler.
//
// This type takes care of actually receiving the events and appending them to an inner buffer so
// they can be used seamlessly by the `MessageHandler`.
struct EventHandler {
    receiver: EventRecv,
    shutdown: ShutdownRecv,
    buffer: Vec<u8>,
    offset: usize,
}

impl EventHandler {
    /// Create a new event handler from an event and a shutdown receivers.
    fn new(receiver: EventRecv, shutdown: ShutdownRecv) -> Self {
        Self {
            receiver,
            shutdown,
            buffer: vec![],
            offset: 0,
        }
    }

    /// Push a new event into the buffer.
    ///
    /// This method also removes the `..self.offset` range from the buffer and sets the offset back
    /// to zero. Which means that this should only be called when the buffer is empty or when there
    /// are not enough bytes to read a new header or payload.
    fn push_event(&mut self, mut bytes: Vec<u8>) {
        // Remove the already read bytes from the buffer.
        self.buffer = self.buffer.split_off(self.offset);
        // Reset the offset.
        self.offset = 0;
        // Append the bytes of the new event
        if self.buffer.is_empty() {
            self.buffer = bytes;
        } else {
            self.buffer.append(&mut bytes);
        }
    }
    /// Fetch a slice of bytes of a determined length.
    ///
    /// This method returns `None` if a shutdown signal was received.
    async fn fetch_bytes<'a>(&'a mut self, len: usize) -> Option<&'a [u8]> {
        // We need to be sure that we have enough bytes in the buffer.
        while self.offset + len > self.buffer.len() {
            // If there are not enough bytes in the buffer, we must either receive new events until
            // we have enough bytes, or receive a shutdown signal.
            select! {
                event = self.receiver.next() => {
                    // If we received an event, we push it to the buffer.
                    if let Some(event) = event {
                        self.push_event(event);
                    }
                },
                _ = &mut self.shutdown => {
                    // If we received a shutdown signal, we return `None`.
                    return None;
                }
            }
        }
        // Get the requested bytes. This will not panic because the loop above only exists if we
        // have enough bytes to do this step.
        let bytes = &self.buffer[self.offset..(self.offset + len)];
        // Increase the offset by the length of the byte slice.
        self.offset += len;
        Some(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use futures::{
        channel::{mpsc, oneshot},
        {future::FutureExt, stream::StreamExt},
    };

    use async_std::task;

    use std::time::Duration;
    /// Generate a vector of events filled with messages of a desired length.
    fn gen_events(event_len: usize, msg_size: usize, n_msg: usize) -> Vec<Vec<u8>> {
        // Bytes of all the messages.
        let mut msgs = vec![0u8; msg_size * n_msg];
        // We need 3 bytes for the header. Thus the message length stored in the header should be 3
        // bytes shorter.
        let msg_len = ((msg_size - 3) as u16).to_be_bytes();
        // We write the bytes that correspond to the message length in the header.
        for i in (0..n_msg).map(|i| i * msg_size + 1) {
            msgs[i] = msg_len[0];
            msgs[i + 1] = msg_len[1];
        }
        // Finally, we split all the bytes into events.
        msgs.chunks(event_len).map(Vec::from).collect()
    }
    /// Test if the `MessageHandler` can produce an exact number of messages of a desired length,
    /// divided in events of an specified length. This test checks that:
    /// - The header and payload of all the messages have the right content.
    /// - The number of produced messages is the desired one.
    async fn test(event_size: usize, msg_size: usize, msg_count: usize) {
        let msg_len = msg_size - 3;
        // Produce the events
        let events = gen_events(event_size, msg_size, msg_count);
        // Create a new message handler
        let (sender_shutdown, receiver_shutdown) = oneshot::channel::<()>();
        let (mut sender, receiver) = mpsc::channel::<Vec<u8>>(9999);
        let mut msg_handler = MessageHandler::new(
            receiver.fuse(),
            receiver_shutdown.fuse(),
            Address::from_addr_str("127.0.0.1:8080").await.unwrap(),
        );
        // Create the task that does the checks of the test.
        let handle = task::spawn(async move {
            // The messages are expected to be filled with zeroes except for the message length
            // field of the header.
            let expected_bytes = vec![0u8; msg_len];
            let expected_msg = (
                Header {
                    message_type: 0,
                    message_length: msg_len as u16,
                },
                expected_bytes.as_slice(),
            );
            // Count how many messages can be fetched.
            let mut counter = 0;
            while let Some(msg) = msg_handler.fetch_message().await {
                // Assert that the messages' content is correct.
                assert_eq!(msg, expected_msg);
                counter += 1;
            }
            // Assert that the number of messages is correct.
            assert_eq!(msg_count, counter);
            // Return back the message handler to avoid dropping the channels.
            msg_handler
        });
        // Send all the events to the message handler.
        for event in events {
            sender.try_send(event).unwrap();
        }
        // Sleep to be sure the handler had time to produce all the messages.
        task::sleep(Duration::from_secs(1)).await;
        // Send a shutdown signal.
        sender_shutdown.send(()).unwrap();
        // Await for the task with the checks to be completed.
        handle.await;
    }
    /// Test that messages are produced correctly when they are divided into one byte events.
    #[async_std::test]
    async fn one_byte_events() {
        test(1, 5, 10).await;
    }
    /// Test that messages are produced correctly when each message fits exactly into an event.
    #[async_std::test]
    async fn one_message_per_event() {
        test(5, 5, 10).await;
    }
    /// Test that messages are produced correctly when two messages fit exactly into an event.
    #[async_std::test]
    async fn two_messages_per_event() {
        test(10, 5, 10).await;
    }
    /// Test that messages are produced correctly when a message fits exactly into two events.
    #[async_std::test]
    async fn two_events_per_message() {
        test(5, 10, 10).await;
    }
    /// Test that messages are produced correctly when a message does not fit in a single event and
    /// it is not aligned either.
    #[async_std::test]
    async fn misaligned_messages() {
        test(3, 5, 10).await;
    }
    /// Test that the handler stops producing messages after receiving the shutdown signal.
    ///
    /// This test is basically the same as the `one_message_per_event` test. But the last event is
    /// sent after the shutdown signal. As a consequence, the last message is not produced by the
    /// message handler.
    #[async_std::test]
    async fn shutdown() {
        let event_size = 5;
        let msg_size = event_size;
        let msg_count = 10;

        let msg_len = msg_size - 3;

        let mut events = gen_events(event_size, msg_size, msg_count);
        // Put the last event into its own variable.
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
            // Assert that we are missing one message
            assert_eq!(msg_count - 1, counter);

            msg_handler
        });

        for event in events {
            sender.try_send(event).unwrap();
        }

        task::sleep(Duration::from_secs(1)).await;
        sender_shutdown.send(()).unwrap();
        // Send the last event after the shutdown signal
        task::sleep(Duration::from_secs(1)).await;
        sender.try_send(last_event).unwrap();

        handle.await;
    }
}
