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

use bee_crypto::SpongeType;
use bee_ternary::{T1B1Buf, T5B1Buf, TryteBuf};
use bee_transaction::{Address, BundledTransactionField};

use bytemuck::cast_slice;
use serde::Deserialize;

const DEFAULT_MWM: u8 = 14;
const DEFAULT_COO_DEPTH: u8 = 24;
const DEFAULT_COO_PUBLIC_KEY: &str =
    "EQSAUZXULTTYZCLNJNTXQTQHOMOFZERHTCGTXOLTVAHKSA9OGAZDEKECURBRIXIJWNPFCQIOVFVVXJVD9";
const DEFAULT_COO_SECURITY: u8 = 2;
const DEFAULT_COO_SPONGE_TYPE: &str = "kerl";
const DEFAULT_MILESTONE_REQUEST_SEND_WORKER_BOUND: usize = 1000;
const DEFAULT_TRANSACTION_BROADCAST_SEND_WORKER_BOUND: usize = 1000;
const DEFAULT_TRANSACTION_REQUEST_SEND_WORKER_BOUND: usize = 1000;
const DEFAULT_HEARTBEAT_SEND_WORKER_BOUND: usize = 1000;
const DEFAULT_MILESTONE_VALIDATOR_WORKER_BOUND: usize = 1000;
const DEFAULT_TRANSACTION_SOLIDIFIER_WORKER_BOUND: usize = 1000;
const DEFAULT_MILESTONE_SOLIDIFIER_WORKER_BOUND: usize = 1000;
const DEFAULT_TRANSACTION_WORKER_BOUND: usize = 1000;
const DEFAULT_TRANSACTION_WORKER_CACHE: usize = 10000;
const DEFAULT_TRANSACTION_RESPONDER_WORKER_BOUND: usize = 1000;
const DEFAULT_MILESTONE_RESPONDER_WORKER_BOUND: usize = 1000;
const DEFAULT_TRANSACTION_REQUESTER_WORKER_BOUND: usize = 1000;
const DEFAULT_MILESTONE_REQUESTER_WORKER_BOUND: usize = 1000;
const DEFAULT_RECEIVER_WORKER_BOUND: usize = 1000;
const DEFAULT_BROADCASTER_WORKER_BOUND: usize = 1000;
const DEFAULT_STATUS_INTERVAL: u64 = 10;
const DEFAULT_HANDSHAKE_WINDOW: u64 = 10;

#[derive(Default, Deserialize)]
struct ProtocolCoordinatorConfigBuilder {
    depth: Option<u8>,
    public_key: Option<String>,
    security_level: Option<u8>,
    sponge_type: Option<String>,
}

#[derive(Default, Deserialize)]
struct ProtocolWorkersConfigBuilder {
    milestone_request_send_worker_bound: Option<usize>,
    transaction_broadcast_send_worker_bound: Option<usize>,
    transaction_request_send_worker_bound: Option<usize>,
    heartbeat_send_worker_bound: Option<usize>,
    milestone_validator_worker_bound: Option<usize>,
    transaction_solidifier_worker_bound: Option<usize>,
    milestone_solidifier_worker_bound: Option<usize>,
    transaction_worker_bound: Option<usize>,
    transaction_worker_cache: Option<usize>,
    transaction_responder_worker_bound: Option<usize>,
    milestone_responder_worker_bound: Option<usize>,
    transaction_requester_worker_bound: Option<usize>,
    milestone_requester_worker_bound: Option<usize>,
    receiver_worker_bound: Option<usize>,
    broadcaster_worker_bound: Option<usize>,
    status_interval: Option<u64>,
}

#[derive(Default, Deserialize)]
pub struct ProtocolConfigBuilder {
    mwm: Option<u8>,
    coordinator: ProtocolCoordinatorConfigBuilder,
    workers: ProtocolWorkersConfigBuilder,
    handshake_window: Option<u64>,
}

impl ProtocolConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mwm(mut self, mwm: u8) -> Self {
        self.mwm.replace(mwm);
        self
    }

    pub fn coo_depth(mut self, coo_depth: u8) -> Self {
        self.coordinator.depth.replace(coo_depth);
        self
    }

    pub fn coo_public_key(mut self, coo_public_key: String) -> Self {
        self.coordinator.public_key.replace(coo_public_key);
        self
    }

    pub fn coo_security_level(mut self, coo_security_level: u8) -> Self {
        self.coordinator.security_level.replace(coo_security_level);
        self
    }

    pub fn coo_sponge_type(mut self, coo_sponge_type: &str) -> Self {
        self.coordinator.sponge_type.replace(coo_sponge_type.to_string());
        self
    }

    pub fn milestone_request_send_worker_bound(mut self, milestone_request_send_worker_bound: usize) -> Self {
        self.workers
            .transaction_broadcast_send_worker_bound
            .replace(milestone_request_send_worker_bound);
        self
    }

    pub fn transaction_broadcast_send_worker_bound(mut self, transaction_broadcast_send_worker_bound: usize) -> Self {
        self.workers
            .transaction_broadcast_send_worker_bound
            .replace(transaction_broadcast_send_worker_bound);
        self
    }

    pub fn transaction_request_send_worker_bound(mut self, transaction_request_send_worker_bound: usize) -> Self {
        self.workers
            .transaction_request_send_worker_bound
            .replace(transaction_request_send_worker_bound);
        self
    }

    pub fn heartbeat_send_worker_bound(mut self, heartbeat_send_worker_bound: usize) -> Self {
        self.workers
            .heartbeat_send_worker_bound
            .replace(heartbeat_send_worker_bound);
        self
    }

    pub fn milestone_validator_worker_bound(mut self, milestone_validator_worker_bound: usize) -> Self {
        self.workers
            .milestone_validator_worker_bound
            .replace(milestone_validator_worker_bound);
        self
    }

    pub fn transaction_solidifier_worker_bound(mut self, transaction_solidifier_worker_bound: usize) -> Self {
        self.workers
            .transaction_solidifier_worker_bound
            .replace(transaction_solidifier_worker_bound);
        self
    }

    pub fn milestone_solidifier_worker_bound(mut self, milestone_solidifier_worker_bound: usize) -> Self {
        self.workers
            .milestone_solidifier_worker_bound
            .replace(milestone_solidifier_worker_bound);
        self
    }

    pub fn transaction_worker_bound(mut self, transaction_worker_bound: usize) -> Self {
        self.workers.transaction_worker_bound.replace(transaction_worker_bound);
        self
    }

    pub fn transaction_worker_cache(mut self, transaction_worker_cache: usize) -> Self {
        self.workers.transaction_worker_cache.replace(transaction_worker_cache);
        self
    }

    pub fn transaction_responder_worker_bound(mut self, transaction_responder_worker_bound: usize) -> Self {
        self.workers
            .transaction_responder_worker_bound
            .replace(transaction_responder_worker_bound);
        self
    }

    pub fn milestone_responder_worker_bound(mut self, milestone_responder_worker_bound: usize) -> Self {
        self.workers
            .milestone_responder_worker_bound
            .replace(milestone_responder_worker_bound);
        self
    }

    pub fn transaction_requester_worker_bound(mut self, transaction_requester_worker_bound: usize) -> Self {
        self.workers
            .transaction_requester_worker_bound
            .replace(transaction_requester_worker_bound);
        self
    }

    pub fn milestone_requester_worker_bound(mut self, milestone_requester_worker_bound: usize) -> Self {
        self.workers
            .milestone_requester_worker_bound
            .replace(milestone_requester_worker_bound);
        self
    }

    pub fn receiver_worker_bound(mut self, receiver_worker_bound: usize) -> Self {
        self.workers.receiver_worker_bound.replace(receiver_worker_bound);
        self
    }

    pub fn broadcaster_worker_bound(mut self, broadcaster_worker_bound: usize) -> Self {
        self.workers.broadcaster_worker_bound.replace(broadcaster_worker_bound);
        self
    }

    pub fn status_interval(mut self, status_interval: u64) -> Self {
        self.workers.status_interval.replace(status_interval);
        self
    }

    pub fn handshake_window(mut self, handshake_window: u64) -> Self {
        self.handshake_window.replace(handshake_window);
        self
    }

    pub fn finish(self) -> ProtocolConfig {
        let coo_sponge_type = match self
            .coordinator
            .sponge_type
            .unwrap_or_else(|| DEFAULT_COO_SPONGE_TYPE.to_owned())
            .as_str()
        {
            "kerl" => SpongeType::Kerl,
            "curl27" => SpongeType::CurlP27,
            "curl81" => SpongeType::CurlP81,
            _ => SpongeType::Kerl,
        };

        let coo_public_key_default = Address::from_inner_unchecked(
            TryteBuf::try_from_str(DEFAULT_COO_PUBLIC_KEY)
                .unwrap()
                .as_trits()
                .encode::<T1B1Buf>(),
        );

        let coo_public_key = match TryteBuf::try_from_str(
            &self
                .coordinator
                .public_key
                .unwrap_or_else(|| DEFAULT_COO_PUBLIC_KEY.to_owned()),
        ) {
            Ok(trytes) => match Address::try_from_inner(trytes.as_trits().encode::<T1B1Buf>()) {
                Ok(coo_public_key) => coo_public_key,
                Err(_) => coo_public_key_default,
            },
            Err(_) => coo_public_key_default,
        };

        let mut public_key_bytes = [0u8; 49];
        public_key_bytes.copy_from_slice(cast_slice(coo_public_key.to_inner().encode::<T5B1Buf>().as_i8_slice()));

        ProtocolConfig {
            null_address: Address::zeros(),
            mwm: self.mwm.unwrap_or(DEFAULT_MWM),
            coordinator: ProtocolCoordinatorConfig {
                depth: self.coordinator.depth.unwrap_or(DEFAULT_COO_DEPTH),
                public_key: coo_public_key,
                public_key_bytes,
                security_level: self.coordinator.security_level.unwrap_or(DEFAULT_COO_SECURITY),
                sponge_type: coo_sponge_type,
            },
            workers: ProtocolWorkersConfig {
                milestone_request_send_worker_bound: self
                    .workers
                    .milestone_request_send_worker_bound
                    .unwrap_or(DEFAULT_MILESTONE_REQUEST_SEND_WORKER_BOUND),
                transaction_broadcast_send_worker_bound: self
                    .workers
                    .transaction_broadcast_send_worker_bound
                    .unwrap_or(DEFAULT_TRANSACTION_BROADCAST_SEND_WORKER_BOUND),
                transaction_request_send_worker_bound: self
                    .workers
                    .transaction_request_send_worker_bound
                    .unwrap_or(DEFAULT_TRANSACTION_REQUEST_SEND_WORKER_BOUND),
                heartbeat_send_worker_bound: self
                    .workers
                    .heartbeat_send_worker_bound
                    .unwrap_or(DEFAULT_HEARTBEAT_SEND_WORKER_BOUND),
                milestone_validator_worker_bound: self
                    .workers
                    .milestone_validator_worker_bound
                    .unwrap_or(DEFAULT_MILESTONE_VALIDATOR_WORKER_BOUND),
                transaction_solidifier_worker_bound: self
                    .workers
                    .transaction_solidifier_worker_bound
                    .unwrap_or(DEFAULT_TRANSACTION_SOLIDIFIER_WORKER_BOUND),
                milestone_solidifier_worker_bound: self
                    .workers
                    .milestone_solidifier_worker_bound
                    .unwrap_or(DEFAULT_MILESTONE_SOLIDIFIER_WORKER_BOUND),
                transaction_worker_bound: self
                    .workers
                    .transaction_worker_bound
                    .unwrap_or(DEFAULT_TRANSACTION_WORKER_BOUND),
                transaction_worker_cache: self
                    .workers
                    .transaction_worker_cache
                    .unwrap_or(DEFAULT_TRANSACTION_WORKER_CACHE),
                transaction_responder_worker_bound: self
                    .workers
                    .transaction_responder_worker_bound
                    .unwrap_or(DEFAULT_TRANSACTION_RESPONDER_WORKER_BOUND),
                milestone_responder_worker_bound: self
                    .workers
                    .milestone_responder_worker_bound
                    .unwrap_or(DEFAULT_MILESTONE_RESPONDER_WORKER_BOUND),
                transaction_requester_worker_bound: self
                    .workers
                    .transaction_requester_worker_bound
                    .unwrap_or(DEFAULT_TRANSACTION_REQUESTER_WORKER_BOUND),
                milestone_requester_worker_bound: self
                    .workers
                    .milestone_requester_worker_bound
                    .unwrap_or(DEFAULT_MILESTONE_REQUESTER_WORKER_BOUND),
                receiver_worker_bound: self
                    .workers
                    .receiver_worker_bound
                    .unwrap_or(DEFAULT_RECEIVER_WORKER_BOUND),
                broadcaster_worker_bound: self
                    .workers
                    .broadcaster_worker_bound
                    .unwrap_or(DEFAULT_BROADCASTER_WORKER_BOUND),
                status_interval: self.workers.status_interval.unwrap_or(DEFAULT_STATUS_INTERVAL),
            },
            handshake_window: self.handshake_window.unwrap_or(DEFAULT_HANDSHAKE_WINDOW),
        }
    }
}

#[derive(Clone)]
pub struct ProtocolCoordinatorConfig {
    pub(crate) depth: u8,
    pub(crate) public_key: Address,
    pub(crate) public_key_bytes: [u8; 49],
    pub(crate) security_level: u8,
    pub(crate) sponge_type: SpongeType,
}

#[derive(Clone)]
pub struct ProtocolWorkersConfig {
    pub(crate) milestone_request_send_worker_bound: usize,
    pub(crate) transaction_broadcast_send_worker_bound: usize,
    pub(crate) transaction_request_send_worker_bound: usize,
    pub(crate) heartbeat_send_worker_bound: usize,
    pub(crate) milestone_validator_worker_bound: usize,
    pub(crate) transaction_solidifier_worker_bound: usize,
    pub(crate) milestone_solidifier_worker_bound: usize,
    pub(crate) transaction_worker_bound: usize,
    pub(crate) transaction_worker_cache: usize,
    pub(crate) transaction_responder_worker_bound: usize,
    pub(crate) milestone_responder_worker_bound: usize,
    pub(crate) transaction_requester_worker_bound: usize,
    pub(crate) milestone_requester_worker_bound: usize,
    pub(crate) receiver_worker_bound: usize,
    pub(crate) broadcaster_worker_bound: usize,
    pub(crate) status_interval: u64,
}

#[derive(Clone)]
pub struct ProtocolConfig {
    pub(crate) null_address: Address,
    pub(crate) mwm: u8,
    pub(crate) coordinator: ProtocolCoordinatorConfig,
    pub(crate) workers: ProtocolWorkersConfig,
    pub(crate) handshake_window: u64,
}

impl ProtocolConfig {
    pub fn build() -> ProtocolConfigBuilder {
        ProtocolConfigBuilder::new()
    }
}

// TODO move out of here
pub(crate) fn slice_eq(a: &[u8; 49], b: &[u8; 49]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    for i in 0..a.len() {
        if a[i] != b[i] {
            return false;
        }
    }

    true
}
