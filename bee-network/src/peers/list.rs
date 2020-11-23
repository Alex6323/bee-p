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

use crate::{Multiaddr, ShortId, KNOWN_PEER_LIMIT, UNKNOWN_PEER_LIMIT};

use super::{errors::Error, DataSender, PeerRelation};

use dashmap::{mapref::entry::Entry, DashMap};
use libp2p::PeerId;

use std::sync::{atomic::Ordering, Arc};

const DEFAULT_PEERLIST_CAPACITY: usize = 8;

#[derive(Clone, Debug)]
pub struct PeerInfo {
    pub address: Multiaddr,
    pub alias: Option<String>,
    pub relation: PeerRelation,
}

#[derive(Clone, Debug)]
pub enum Repeat {
    Times(usize),
    Forever,
}

#[derive(Clone, Debug)]
pub enum PeerState {
    Disconnected,
    // Note: forever trying if `None`
    Connecting { remaining_attempts: Repeat },
    Connected(DataSender),
}

impl PeerState {
    pub fn is_connected(&self) -> bool {
        if let PeerState::Connected(_) = *self {
            true
        } else {
            false
        }
    }

    pub fn is_connecting(&self) -> bool {
        if let PeerState::Connecting { remaining_attempts: _ } = *self {
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct PeerList(Arc<DashMap<PeerId, (PeerInfo, PeerState)>>);

impl PeerList {
    pub fn new() -> Self {
        Self(Arc::new(DashMap::with_capacity(DEFAULT_PEERLIST_CAPACITY)))
    }

    // If the insertion fails for some reason, we give it back to the caller.
    pub fn insert(&self, id: PeerId, info: PeerInfo) -> Result<(), (PeerInfo, Error)> {
        match self.0.entry(id) {
            Entry::Occupied(entry) => return Err((info, Error::PeerAlreadyAdded(entry.key().short()))),
            Entry::Vacant(entry) => match info.relation {
                PeerRelation::Known => {
                    if self.num_known() < KNOWN_PEER_LIMIT.load(Ordering::Relaxed) {
                        entry.insert((info, PeerState::Disconnected));
                    } else {
                        return Err((
                            info,
                            Error::KnownPeerLimitReached(KNOWN_PEER_LIMIT.load(Ordering::Relaxed)),
                        ));
                    }
                }
                PeerRelation::Unknown => {
                    if self.num_unknown() < UNKNOWN_PEER_LIMIT.load(Ordering::Relaxed) {
                        entry.insert((info, PeerState::Disconnected));
                    } else {
                        return Err((
                            info,
                            Error::UnknownPeerLimitReached(UNKNOWN_PEER_LIMIT.load(Ordering::Relaxed)),
                        ));
                    }
                }
                PeerRelation::Discovered => {
                    todo!("PeerRelation::Discovered case");
                }
            },
        }
        Ok(())
    }

    pub fn update_relation(&self, id: &PeerId, relation: PeerRelation) -> bool {
        self.0
            .get_mut(id)
            .map(|mut kv| kv.value_mut().0.relation = relation)
            .is_some()
    }

    pub fn update_state(&self, id: &PeerId, state: PeerState) -> bool {
        self.0.get_mut(id).map(|mut kv| kv.value_mut().1 = state).is_some()
    }

    pub fn contains(&self, id: &PeerId) -> bool {
        self.0.contains_key(id)
    }

    pub fn remove(&self, id: &PeerId) -> Result<PeerInfo, Error> {
        if let Some((_, (peer_info, _))) = self.0.remove(id) {
            Ok(peer_info)
        } else {
            Err(Error::PeerAlreadyRemoved(id.short()))
        }
    }

    pub fn get_info(&self, id: &PeerId) -> Option<PeerInfo> {
        self.0.get(id).map(|kv| kv.value().0.clone())
    }

    pub async fn send_message(&self, message: Vec<u8>, to: &PeerId) -> Result<(), Error> {
        if let Some(mut kv) = self.0.get_mut(to) {
            if let PeerState::Connected(sender) = &mut kv.value_mut().1 {
                sender
                    .send_async(message)
                    .await
                    .map_err(|_| Error::SendMessageFailure(to.short()))?;

                Ok(())
            } else {
                Err(Error::DisconnectedPeer(to.short()))
            }
        } else {
            Err(Error::UnlistedPeer(to.short()))
        }
    }

    pub fn size(&self) -> usize {
        self.0.len()
    }

    pub fn is_connected(&self, id: &PeerId) -> bool {
        self.0.get(id).map(|kv| kv.value().1.is_connected()).is_some()
    }

    pub fn is_connecting(&self, id: &PeerId) -> bool {
        self.0.get(id).map(|kv| kv.value().1.is_connecting()).is_some()
    }

    pub fn num_connected(&self) -> usize {
        self.0.iter().fold(
            0,
            |count, kv| if kv.value().1.is_connected() { count + 1 } else { count },
        )
    }
}

macro_rules! impl_relation_iter {
    ($type:tt, $is:tt, $iter:tt, $num:tt) => {
        pub struct $type {
            peer_list: PeerList,
            start: usize,
        }

        impl Iterator for $type {
            type Item = PeerId;

            fn next(&mut self) -> Option<Self::Item> {
                let mut new_start = 0;
                let result = self
                    .peer_list
                    .0
                    .iter()
                    .skip(self.start)
                    .enumerate()
                    .find_map(|(pos, kv)| {
                        if kv.value().0.relation.$is() {
                            new_start = pos;
                            Some(kv.key().clone())
                        } else {
                            None
                        }
                    });
                self.start = new_start;
                result
            }
        }

        impl PeerList {
            pub fn $iter(&self) -> $type {
                $type {
                    peer_list: self.clone(),
                    start: 0,
                }
            }

            pub fn $is(&self, peer_id: &PeerId) -> bool {
                self.$iter().find(|id| id == peer_id).is_some()
            }

            pub fn $num(&self) -> usize {
                self.0.iter().fold(0, |count, kv| {
                    if kv.value().0.relation.$is() {
                        count + 1
                    } else {
                        count
                    }
                })
            }
        }

        impl PeerInfo {
            pub fn $is(&self) -> bool {
                self.relation.$is()
            }
        }
    };
}

impl_relation_iter!(KnownPeersIter, is_known, iter_known, num_known);
impl_relation_iter!(UnknownPeersIter, is_unknown, iter_unknown, num_unknown);
impl_relation_iter!(DiscoveredPeersIter, is_discovered, iter_discovered, num_discovered);
