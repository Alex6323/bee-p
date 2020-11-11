use crate::PeerId;

use thiserror::Error as ErrorAttr;

#[derive(Debug, ErrorAttr)]
pub enum Error {
    #[error("Failed to send an event ({}).", .0)]
    EventSendFailure(&'static str),
    #[error("Failed to send an internal event ({}).", .0)]
    InternalEventSendFailure(&'static str),
    #[error("Failed to send a message to {}", .0)]
    SendMessageFailure(PeerId),
    #[error("Unknown peer: {}", .0)]
    UnknownPeer(PeerId),
    #[error("Disconnected peer: {}", .0)]
    DisconnectedPeer(PeerId),
    #[error("Failed to disconnect from peer: {}", .0)]
    DisconnectPeerFailure(PeerId),
}
