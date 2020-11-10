use futures::prelude::*;
use libp2p::core::{
    muxing::{StreamMuxerBox, SubstreamRef},
    InboundUpgrade, OutboundUpgrade, UpgradeInfo,
};

use std::{io, iter, sync::Arc};

type NegotiatedSubstream = SubstreamRef<Arc<StreamMuxerBox>>;

#[derive(Default, Debug, Copy, Clone)]
pub struct GossipProtocol;

impl UpgradeInfo for GossipProtocol {
    type Info = &'static [u8];
    type InfoIter = iter::Once<Self::Info>;

    fn protocol_info(&self) -> Self::InfoIter {
        iter::once(b"/iota-gossip/1/1.0.0")
    }
}

impl InboundUpgrade<NegotiatedSubstream> for GossipProtocol {
    type Output = NegotiatedSubstream;
    type Error = (); // Void;
    type Future = future::Ready<Result<Self::Output, Self::Error>>;

    fn upgrade_inbound(self, stream: SubstreamRef<Arc<StreamMuxerBox>>, _: Self::Info) -> Self::Future {
        future::ok(stream)
    }
}

impl OutboundUpgrade<NegotiatedSubstream> for GossipProtocol {
    type Output = NegotiatedSubstream;
    type Error = (); // Void;
    type Future = future::Ready<Result<Self::Output, Self::Error>>;

    fn upgrade_outbound(self, stream: NegotiatedSubstream, _: Self::Info) -> Self::Future {
        future::ok(stream)
    }
}

// pub async fn send_message<S>(mut stream: S) -> io::Result<(S, Duration)>
// where
//     S: AsyncRead + AsyncWrite + Unpin,
// {
//     let payload: [u8; PING_SIZE] = thread_rng().sample(distributions::Standard);
//     log::debug!("Preparing ping payload {:?}", payload);
//     stream.write_all(&payload).await?;
//     stream.flush().await?;
//     let started = Instant::now();
//     let mut recv_payload = [0u8; PING_SIZE];
//     log::debug!("Awaiting pong for {:?}", payload);
//     stream.read_exact(&mut recv_payload).await?;
//     if recv_payload == payload {
//         Ok((stream, started.elapsed()))
//     } else {
//         Err(io::Error::new(io::ErrorKind::InvalidData, "Ping payload mismatch"))
//     }
// }

// pub async fn recv_message<S>(mut stream: S) -> io::Result<S>
// where
//     S: AsyncRead + AsyncWrite + Unpin,
// {
//     let mut payload = [0u8; PING_SIZE];
//     log::debug!("Waiting for ping ...");
//     stream.read_exact(&mut payload).await?;
//     log::debug!("Sending pong for {:?}", payload);
//     stream.write_all(&payload).await?;
//     stream.flush().await?;
//     Ok(stream)
// }
