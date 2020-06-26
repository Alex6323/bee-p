use crate::peer::{Peer, PeerId};

use futures::{channel::mpsc, stream::StreamExt};

const EVENT_CHANNEL_CAPACITY: usize = 10000;

#[non_exhaustive]
pub enum Event {
    /// Triggered, wehen a new peer has been discovered and verified.
    PeerDiscovered { peer: Peer },

    /// Triggered, wehen a discovered and verified peer could not be re-verified.
    PeerDeleted { peer: PeerId },
}

pub struct EventSender(mpsc::Sender<Event>);

impl std::ops::Deref for EventSender {
    type Target = mpsc::Sender<Event>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct EventReceiver(futures::stream::Fuse<mpsc::Receiver<Event>>);

impl std::ops::Deref for EventReceiver {
    type Target = futures::stream::Fuse<mpsc::Receiver<Event>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn event_channel() -> (EventSender, EventReceiver) {
    let (sender, receiver) = mpsc::channel(EVENT_CHANNEL_CAPACITY);
    (EventSender(sender), EventReceiver(receiver.fuse()))
}
