// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use libp2p::{
    core::{
        muxing::StreamMuxerBox,
        transport::{upgrade, Boxed},
        upgrade::SelectUpgrade,
    },
    dns, identity, mplex, noise, tcp, yamux, PeerId, Transport,
};

use std::io;

// let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
//     .into_authentic(local_keys)
//     .expect("error creating noise keys");

// let transport = tcp::TokioTcpConfig::default()
//     .upgrade(upgrade::Version::V1)
//     .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
//     .multiplex(yamux::Config::default())
//     .map(|(peer, muxer), _| (peer, StreamMuxerBox::new(muxer)))
//     .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
//     .boxed();

pub fn build_transport(local_keys: &identity::Keypair) -> io::Result<Boxed<(PeerId, StreamMuxerBox)>> {
    let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
        .into_authentic(local_keys)
        .expect("error creating noise keys");

    let tcp = tcp::TokioTcpConfig::new().nodelay(true);
    let transport = dns::DnsConfig::new(tcp)?;

    Ok(transport
        .upgrade(upgrade::Version::V1)
        .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
        // .multiplex(yamux::Config::default())
        .multiplex(SelectUpgrade::new(yamux::Config::default(), mplex::MplexConfig::new()))
        .timeout(std::time::Duration::from_secs(20))
        // .map(|(peer, muxer), _| (peer, StreamMuxerBox::new(muxer)))
        // .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
        .boxed())
}
