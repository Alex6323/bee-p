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
    types::{DataResponse, GetInfoResponseBody, GetMilestoneByIndexResponseBody, GetTipsResponseBody, *},
};
use bee_common_ext::node::ResHandle;
use bee_message::{payload::milestone::MilestoneEssence, prelude::*};
use bee_protocol::{tangle::MsTangle, MilestoneIndex};
use bee_storage::storage::Backend;
use std::{
    convert::Infallible,
    time::{SystemTime, UNIX_EPOCH},
};
use warp::{http::StatusCode, reject, Rejection, Reply};

async fn is_healthy<B: Backend>(tangle: ResHandle<MsTangle<B>>) -> bool {
    let mut is_healthy = true;

    if !tangle.is_synced() {
        is_healthy = false;
    }

    // TODO: check if number of peers != 0

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
    // TODO: get public key of coordinator from protocol config
    let coordinator_public_key = String::from("52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649");
    let latest_milestone_message_id = tangle
        .get_milestone_message_id(tangle.get_latest_milestone_index())
        .unwrap()
        .to_string();
    let latest_milestone_index = *tangle.get_latest_milestone_index();
    let solid_milestone_message_id = tangle
        .get_milestone_message_id(tangle.get_latest_milestone_index())
        .unwrap()
        .to_string();
    let solid_milestone_index = *tangle.get_latest_milestone_index();
    let pruning_index = *tangle.get_pruning_index();
    // TODO: check enabled features
    let features = Vec::new();

    Ok(warp::reply::json(&DataResponse::new(GetInfoResponseBody {
        name,
        version,
        is_healthy,
        coordinator_public_key,
        latest_milestone_message_id,
        latest_milestone_index,
        solid_milestone_message_id,
        solid_milestone_index,
        pruning_index,
        features,
    })))
}

pub async fn get_tips<B: Backend>(tangle: ResHandle<MsTangle<B>>) -> Result<impl Reply, Rejection> {
    match tangle.get_messages_to_approve().await {
        Some(tips) => Ok(warp::reply::json(&DataResponse::new(GetTipsResponseBody {
            tip_1_message_id: tips.0.to_string(),
            tip_2_message_id: tips.1.to_string(),
        }))),
        None => Err(reject::custom(ServiceUnavailable)),
    }
}

pub async fn get_message_by_id<B: Backend>(
    message_id: MessageId,
    tangle: ResHandle<MsTangle<B>>,
) -> Result<impl Reply, Rejection> {
    match tangle.get(&message_id).await {
        Some(message) => {
            let version = 1;
            let parent_1_message_id = message.parent1().to_string();
            let parent_2_message_id = message.parent2().to_string();
            let payload = {
                match message.payload() {
                    Payload::Transaction(t) => PayloadDto::Transaction(TransactionPayloadDto {
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
                                })
                                .collect(),
                            outputs: t
                                .essence()
                                .outputs()
                                .iter()
                                .map(|output| match output {
                                    Output::SignatureLockedSingle(output) => SigLockedSingleOutputDto {
                                        kind: 0,
                                        address: match output.address() {
                                            Address::Ed25519(ed) => Ed25519AddressDto {
                                                kind: 1,
                                                address: ed.to_string(),
                                            },
                                            Address::Wots(_) => unimplemented!(),
                                        },
                                        amount: 0,
                                    },
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
                                    SignatureUnlock::Wots(_) => unimplemented!(),
                                },
                                UnlockBlock::Reference(r) => UnlockBlockDto::Reference(ReferenceUnlockBlockDto {
                                    kind: 1,
                                    reference: r.index(),
                                }),
                            })
                            .collect(),
                    }),
                    Payload::Milestone(m) => PayloadDto::Milestone(MilestonePayloadDto {
                        kind: 1,
                        index: m.essence().index(),
                        timestamp: m.essence().timestamp(),
                        inclusion_merkle_proof: hex::encode(m.essence().merkle_proof()),
                        signatures: m.signatures().iter().map(|sig| hex::encode(sig)).collect(),
                    }),
                    Payload::Indexation(i) => PayloadDto::Indexation(IndexationPayloadDto {
                        kind: 2,
                        index: i.index().to_owned(),
                        data: hex::encode(i.data()),
                    }),
                }
            };
            let nonce = message.nonce();

            Ok(warp::reply::json(&DataResponse::new(GetMessageByIdResponseBody(
                MessageDto {
                    version,
                    parent_1_message_id,
                    parent_2_message_id,
                    payload,
                    nonce,
                },
            ))))
        }
        None => Err(reject::not_found()),
    }
}

pub async fn get_milestone_by_index<B: Backend>(
    milestone_index: MilestoneIndex,
    tangle: ResHandle<MsTangle<B>>,
) -> Result<impl Reply, Rejection> {
    match tangle.get_milestone_message_id(milestone_index) {
        Some(message_id) => match tangle.get_metadata(&message_id) {
            Some(metadata) => {
                let timestamp = metadata.arrival_timestamp();
                Ok(warp::reply::json(&DataResponse::new(GetMilestoneByIndexResponseBody {
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

pub mod tests {

    use super::*;

    pub fn indexation_message() -> Message {
        Message::builder()
            .with_parent1(MessageId::new([
                0xF5, 0x32, 0xA5, 0x35, 0x45, 0x10, 0x32, 0x76, 0xB4, 0x68, 0x76, 0xC4, 0x73, 0x84, 0x6D, 0x98, 0x64,
                0x8E, 0xE4, 0x18, 0x46, 0x8B, 0xCE, 0x76, 0xDF, 0x48, 0x68, 0x64, 0x8D, 0xD7, 0x3E, 0x5D,
            ]))
            .with_parent2(MessageId::new([
                0x78, 0xD5, 0x46, 0xB4, 0x6A, 0xEC, 0x45, 0x57, 0x87, 0x21, 0x39, 0xA4, 0x8F, 0x66, 0xBC, 0x56, 0x76,
                0x87, 0xE8, 0x41, 0x35, 0x78, 0xA1, 0x43, 0x23, 0x54, 0x87, 0x32, 0x35, 0x89, 0x14, 0xA2,
            ]))
            .with_payload(Payload::Indexation(Box::new(Indexation::new(
                "MYINDEX".to_owned(),
                Box::new([0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x49, 0x6f, 0x74, 0x61]),
            ))))
            .finish()
            .unwrap()
    }

    pub fn milestone_message() -> Message {
        Message::builder()
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
