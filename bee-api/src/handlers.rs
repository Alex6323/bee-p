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

use crate::{
    filters::ServiceUnavailable,
    storage::Backend,
    types::{DataResponse, GetInfoResponse, GetMilestoneResponse, GetTipsResponse, *},
};
use bee_common::packable::Packable;
use bee_common_ext::node::ResHandle;
use bee_ledger::spent::Spent;
use bee_message::{payload::milestone::MilestoneEssence, prelude::*};
use bee_protocol::{tangle::MsTangle, MilestoneIndex};
use blake2::Blake2s;
use digest::Digest;
use std::{
    convert::{Infallible, TryInto},
    iter::FromIterator,
    ops::Deref,
    time::{SystemTime, UNIX_EPOCH},
};
use warp::{
    http::{Response, StatusCode},
    reject, Rejection, Reply,
};

async fn is_healthy<B: Backend>(tangle: ResHandle<MsTangle<B>>) -> bool {
    let mut is_healthy = true;

    if !tangle.is_synced() {
        is_healthy = false;
    }

    // TODO: check if number of peers != 0 else return false

    match tangle.get_milestone_message_id(tangle.get_latest_milestone_index()) {
        Some(milestone_message_id) => match tangle.get_metadata(&milestone_message_id) {
            Some(metadata) => {
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Clock may have gone backwards")
                    .as_millis() as u64;
                let latest_milestone_arrival_timestamp = metadata.arrival_timestamp();
                if current_time - latest_milestone_arrival_timestamp > 5 * 60 * 60000 {
                    is_healthy = false;
                }
            }
            None => is_healthy = false,
        },
        None => is_healthy = false,
    }

    is_healthy
}

pub async fn get_health<B: Backend>(tangle: ResHandle<MsTangle<B>>) -> Result<impl Reply, Infallible> {
    if is_healthy(tangle).await {
        Ok(StatusCode::OK)
    } else {
        Ok(StatusCode::SERVICE_UNAVAILABLE)
    }
}

pub async fn get_info<B: Backend>(tangle: ResHandle<MsTangle<B>>) -> Result<impl Reply, Infallible> {
    let name = String::from("Bee");
    let version = String::from(env!("CARGO_PKG_VERSION"));
    let is_healthy = is_healthy(tangle.clone()).await;
    // TODO: get network_id from node; use a splaceholder for now
    let network_id = 1;
    let latest_milestone_id = tangle
        .get_milestone_message_id(tangle.get_latest_milestone_index())
        .unwrap()
        .to_string();
    let latest_milestone_index = *tangle.get_latest_milestone_index();
    let solid_milestone_id = tangle
        .get_milestone_message_id(tangle.get_latest_milestone_index())
        .unwrap()
        .to_string();
    let solid_milestone_index = *tangle.get_latest_milestone_index();
    let pruning_index = *tangle.get_pruning_index();
    // TODO: check enabled features
    let features = Vec::new();

    Ok(warp::reply::json(&DataResponse::new(GetInfoResponse {
        name,
        version,
        is_healthy,
        network_id,
        latest_milestone_id,
        latest_milestone_index,
        solid_milestone_id,
        solid_milestone_index,
        pruning_index,
        features,
    })))
}

pub async fn get_tips<B: Backend>(tangle: ResHandle<MsTangle<B>>) -> Result<impl Reply, Rejection> {
    match tangle.get_messages_to_approve().await {
        Some(tips) => Ok(warp::reply::json(&DataResponse::new(GetTipsResponse {
            tip_1_message_id: tips.0.to_string(),
            tip_2_message_id: tips.1.to_string(),
        }))),
        None => Err(reject::custom(ServiceUnavailable)),
    }
}

pub async fn get_message_by_index<B: Backend>(index: String, storage: ResHandle<B>) -> Result<impl Reply, Rejection> {
    let mut hasher = Blake2s::new();
    hasher.update(index.as_bytes());
    let hashed_index = HashedIndex::new(hasher.finalize_reset().as_slice().try_into().unwrap());

    let max_results = 1000;
    match storage.deref().fetch(&hashed_index).await {
        Ok(ret) => match ret {
            Some(mut fetched) => {
                let count = fetched.len();
                fetched.truncate(max_results);
                Ok(warp::reply::json(&DataResponse::new(GetMessagesByIndexResponse {
                    index,
                    max_results,
                    count,
                    message_ids: fetched.iter().map(|id| id.to_string()).collect(),
                })))
            }
            None => Ok(warp::reply::json(&DataResponse::new(GetMessagesByIndexResponse {
                index,
                max_results,
                count: 0,
                message_ids: vec![],
            }))),
        },
        Err(_) => Err(reject::custom(ServiceUnavailable)),
    }
}

pub async fn get_message_by_message_id<B: Backend>(
    message_id: MessageId,
    tangle: ResHandle<MsTangle<B>>,
) -> Result<impl Reply, Rejection> {
    match tangle.get(&message_id).await {
        Some(message) => {
            // let network_id = message.network_id().to_string();
            let parent_1_message_id = message.parent1().to_string();
            let parent_2_message_id = message.parent2().to_string();
            let payload = {
                match message.payload() {
                    Some(Payload::Transaction(t)) => Some(PayloadDto::Transaction(TransactionPayloadDto {
                        kind: 0,
                        essence: TransactionEssenceDto {
                            kind: 0,
                            inputs: t
                                .essence()
                                .inputs()
                                .iter()
                                .map(|input| match input {
                                    Input::UTXO(input) => UtxoInputDto {
                                        kind: 0,
                                        transaction_id: input.output_id().transaction_id().to_string(),
                                        transaction_output_index: input.output_id().index(),
                                    },
                                    _ => unimplemented!(),
                                })
                                .collect(),
                            outputs: t
                                .essence()
                                .outputs()
                                .iter()
                                .map(|output| match output {
                                    Output::SignatureLockedSingle(output) => {
                                        OutputDto::SignatureLockedSingle(SignatureLockedSingleOutputDto {
                                            kind: 0,
                                            address: match output.address() {
                                                Address::Ed25519(ed) => Ed25519AddressDto {
                                                    kind: 1,
                                                    address: ed.to_bech32(),
                                                },
                                                _ => unimplemented!(),
                                            },
                                            amount: output.amount().to_string(),
                                        })
                                    }
                                    _ => unimplemented!(),
                                })
                                .collect(),
                            payload: match t.essence().payload() {
                                Some(Payload::Indexation(i)) => Some(IndexationPayloadDto {
                                    kind: 2,
                                    index: i.index().to_owned(),
                                    data: hex::encode(i.data()),
                                }),
                                Some(_) => unreachable!(),
                                None => None,
                            },
                        },
                        unlock_blocks: t
                            .unlock_blocks()
                            .iter()
                            .map(|unlock_block| match unlock_block {
                                UnlockBlock::Signature(s) => match s {
                                    SignatureUnlock::Ed25519(ed) => {
                                        UnlockBlockDto::Signature(SignatureUnlockBlockDto {
                                            kind: 1,
                                            signature: crate::types::Ed25519SignatureDto {
                                                kind: 0,
                                                public_key: hex::encode(ed.public_key()),
                                                signature: hex::encode(ed.signature()),
                                            },
                                        })
                                    }
                                    _ => unimplemented!(),
                                },
                                UnlockBlock::Reference(r) => UnlockBlockDto::Reference(ReferenceUnlockBlockDto {
                                    kind: 1,
                                    reference: r.index(),
                                }),
                                _ => unimplemented!(),
                            })
                            .collect(),
                    })),
                    Some(Payload::Milestone(m)) => Some(PayloadDto::Milestone(MilestonePayloadDto {
                        kind: 1,
                        index: m.essence().index(),
                        timestamp: m.essence().timestamp(),
                        inclusion_merkle_proof: hex::encode(m.essence().merkle_proof()),
                        signatures: m.signatures().iter().map(|sig| hex::encode(sig)).collect(),
                    })),
                    Some(Payload::Indexation(i)) => Some(PayloadDto::Indexation(IndexationPayloadDto {
                        kind: 2,
                        index: i.index().to_owned(),
                        data: hex::encode(i.data()),
                    })),
                    Some(_) => unimplemented!(),
                    None => None,
                }
            };
            let nonce = message.nonce().to_string();

            Ok(warp::reply::json(&DataResponse::new(GetMessageResponse(MessageDto {
                // network_id,
                parent_1_message_id,
                parent_2_message_id,
                payload,
                nonce,
            }))))
        }
        None => Err(reject::not_found()),
    }
}

pub async fn get_raw_message<B: Backend>(
    message_id: MessageId,
    tangle: ResHandle<MsTangle<B>>,
) -> Result<impl Reply, Rejection> {
    match tangle.get(&message_id).await {
        Some(message) => Ok(Response::builder()
            .header("Content-Type", "application/octet-stream")
            .body(message.pack_new())),
        None => Err(reject::not_found()),
    }
}

pub async fn get_children_by_message_id<B: Backend>(
    message_id: MessageId,
    tangle: ResHandle<MsTangle<B>>,
) -> Result<impl Reply, Rejection> {
    if tangle.contains(&message_id).await {
        let max_results = 1000;
        let mut children = Vec::from_iter(tangle.get_children(&message_id));
        let count = children.len();
        children.truncate(max_results);
        Ok(warp::reply::json(&DataResponse::new(GetChildrenResponse {
            message_id: message_id.to_string(),
            max_results,
            count,
            children_message_ids: children.iter().map(|id| id.to_string()).collect(),
        })))
    } else {
        Err(reject::not_found())
    }
}

pub async fn get_milestone_by_milestone_index<B: Backend>(
    milestone_index: MilestoneIndex,
    tangle: ResHandle<MsTangle<B>>,
) -> Result<impl Reply, Rejection> {
    match tangle.get_milestone_message_id(milestone_index) {
        Some(message_id) => match tangle.get_metadata(&message_id) {
            Some(metadata) => {
                let timestamp = metadata.arrival_timestamp();
                Ok(warp::reply::json(&DataResponse::new(GetMilestoneResponse {
                    milestone_index: *milestone_index,
                    message_id: message_id.to_string(),
                    timestamp,
                })))
            }
            None => Err(reject::not_found()),
        },
        None => Err(reject::not_found()),
    }
}

pub async fn get_output_by_output_id<B: Backend>(
    output_id: OutputId,
    storage: ResHandle<B>,
) -> Result<impl Reply, Rejection> {
    let output: Result<Option<bee_ledger::output::Output>, <B as bee_storage::storage::Backend>::Error> =
        storage.fetch(&output_id).await;
    let is_spent: Result<Option<Spent>, <B as bee_storage::storage::Backend>::Error> = storage.fetch(&output_id).await;

    if let (Ok(output), Ok(is_spent)) = (output, is_spent) {
        match output {
            Some(output) => Ok(warp::reply::json(&DataResponse::new(GetOutputByOutputIdResponse {
                message_id: output.message_id().to_string(),
                transaction_id: output_id.transaction_id().to_string(),
                output_index: output_id.index(),
                is_spent: is_spent.is_some(),
                output: match output.inner() {
                    Output::SignatureLockedSingle(output) => {
                        OutputDto::SignatureLockedSingle(SignatureLockedSingleOutputDto {
                            kind: 0,
                            address: Ed25519AddressDto {
                                kind: 1,
                                address: output.address().to_bech32(),
                            },
                            amount: output.amount().to_string(),
                        })
                    }
                    _ => panic!("unexpected signature scheme"),
                },
            }))),
            None => Err(reject::not_found()),
        }
    } else {
        Err(reject::custom(ServiceUnavailable))
    }
}

pub async fn get_balance_by_address<B: Backend>(
    output_id: OutputId,
    storage: ResHandle<B>,
) -> Result<impl Reply, Rejection> {
    Ok(StatusCode::OK)
}

pub mod tests {

    use super::*;

    pub fn message_without_payload() -> Message {
        Message::builder()
            //.with_network_id(1)
            .with_parent1(MessageId::new([
                0xF5, 0x32, 0xA5, 0x35, 0x45, 0x10, 0x32, 0x76, 0xB4, 0x68, 0x76, 0xC4, 0x73, 0x84, 0x6D, 0x98, 0x64,
                0x8E, 0xE4, 0x18, 0x46, 0x8B, 0xCE, 0x76, 0xDF, 0x48, 0x68, 0x64, 0x8D, 0xD7, 0x3E, 0x5D,
            ]))
            .with_parent2(MessageId::new([
                0x78, 0xD5, 0x46, 0xB4, 0x6A, 0xEC, 0x45, 0x57, 0x87, 0x21, 0x39, 0xA4, 0x8F, 0x66, 0xBC, 0x56, 0x76,
                0x87, 0xE8, 0x41, 0x35, 0x78, 0xA1, 0x43, 0x23, 0x54, 0x87, 0x32, 0x35, 0x89, 0x14, 0xA2,
            ]))
            .finish()
            .unwrap()
    }

    pub fn indexation_message() -> Message {
        Message::builder()
            //.with_network_id(1)
            .with_parent1(MessageId::new([
                0xF5, 0x32, 0xA5, 0x35, 0x45, 0x10, 0x32, 0x76, 0xB4, 0x68, 0x76, 0xC4, 0x73, 0x84, 0x6D, 0x98, 0x64,
                0x8E, 0xE4, 0x18, 0x46, 0x8B, 0xCE, 0x76, 0xDF, 0x48, 0x68, 0x64, 0x8D, 0xD7, 0x3E, 0x5D,
            ]))
            .with_parent2(MessageId::new([
                0x78, 0xD5, 0x46, 0xB4, 0x6A, 0xEC, 0x45, 0x57, 0x87, 0x21, 0x39, 0xA4, 0x8F, 0x66, 0xBC, 0x56, 0x76,
                0x87, 0xE8, 0x41, 0x35, 0x78, 0xA1, 0x43, 0x23, 0x54, 0x87, 0x32, 0x35, 0x89, 0x14, 0xA2,
            ]))
            .with_payload(Payload::Indexation(Box::new(
                Indexation::new(
                    "MYINDEX".to_owned(),
                    &[0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x49, 0x6f, 0x74, 0x61],
                )
                .unwrap(),
            )))
            .finish()
            .unwrap()
    }

    pub fn milestone_message() -> Message {
        Message::builder()
            //.with_network_id(1)
            .with_parent1(MessageId::new([
                0xF5, 0x32, 0xA5, 0x35, 0x45, 0x10, 0x32, 0x76, 0xB4, 0x68, 0x76, 0xC4, 0x73, 0x84, 0x6D, 0x98, 0x64,
                0x8E, 0xE4, 0x18, 0x46, 0x8B, 0xCE, 0x76, 0xDF, 0x48, 0x68, 0x64, 0x8D, 0xD7, 0x3E, 0x5D,
            ]))
            .with_parent2(MessageId::new([
                0x78, 0xD5, 0x46, 0xB4, 0x6A, 0xEC, 0x45, 0x57, 0x87, 0x21, 0x39, 0xA4, 0x8F, 0x66, 0xBC, 0x56, 0x76,
                0x87, 0xE8, 0x41, 0x35, 0x78, 0xA1, 0x43, 0x23, 0x54, 0x87, 0x32, 0x35, 0x89, 0x14, 0xA2,
            ]))
            .with_payload(Payload::Milestone(Box::new(Milestone::new(
                MilestoneEssence::new(
                    1633,
                    1604072711,
                    MessageId::new([
                        0x78, 0xD5, 0x46, 0xB4, 0x6A, 0xEC, 0x45, 0x57, 0x87, 0x21, 0x39, 0xA4, 0x8F, 0x66, 0xBC, 0x56, 0x76,
                        0x87, 0xE8, 0x41, 0x35, 0x78, 0xA1, 0x43, 0x23, 0x54, 0x87, 0x32, 0x35, 0x89, 0x14, 0xA2,
                    ]),
                    MessageId::new([
                        0x78, 0xD5, 0x46, 0xB4, 0x6A, 0xEC, 0x45, 0x57, 0x87, 0x21, 0x39, 0xA4, 0x8F, 0x66, 0xBC, 0x56, 0x76,
                        0x87, 0xE8, 0x41, 0x35, 0x78, 0xA1, 0x43, 0x23, 0x54, 0x87, 0x32, 0x35, 0x89, 0x14, 0xA2,
                    ]),
                    hex::decode("786a02f742015903c6c6fd852552d272912f4740e15847618a86e217f71f5419d25e1031afee585313896444934eb04b903a685b1448b755d56f701afe9be2ce").unwrap().into_boxed_slice(),

                    vec![[0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x49, 0x6f,0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x49, 0x6f,0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x49, 0x6f,0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x49, 0x6f,]],
                ),
                vec![
                    hex::decode("a3676743c128a78323598965ef89c43ab412e207083feb80fbb3e3a4327aa4bb161f7be427641a21b23af9a58c5a0efdd36f26b2af893e7ad899b76f19cc410d").unwrap().into_boxed_slice(),
                    hex::decode("b3676743c128a78323598965ef89c43ab412e207083feb80fbb3e3a4327aa4bb161f7be427641a21b23af9a58c5a0efdd36f26b2af893e7ad899b76f19cc410d").unwrap().into_boxed_slice(),
                ]
            )))).finish()
            .unwrap()
    }
}
