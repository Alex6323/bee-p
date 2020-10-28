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

use bee_common_ext::node::ResHandle;
use bee_message::MessageId;
use bee_protocol::tangle::MsTangle;
use bee_storage::storage::Backend;
use std::{
    convert::Infallible,
    time::{SystemTime, UNIX_EPOCH},
};
use warp::reply::json;
use bee_protocol::MilestoneIndex;
use crate::dto::DataResponse;
use crate::dto::get_info::GetInfoResponseBody;
use bee_tangle::MessageRef;
use crate::dto::get_milestones::GetMilestonesResponseBody;
use std::sync::Arc;
use warp::{Rejection, reject, Reply};

pub async fn get_info<B: Backend>(tangle: ResHandle<MsTangle<B>>) -> Result<impl Reply, Infallible> {
    let name = String::from("Bee");
    let version = String::from(env!("CARGO_PKG_VERSION"));
    let is_healthy = {
        let mut ret = true;

        if !tangle.is_synced() {
            ret = false;
        }

        // TODO: check if number of peers != 0

        match tangle.get_milestone_message_id(tangle.get_latest_milestone_index()) {
            Some(milestone_message_id) => {
                match tangle.get_metadata(&milestone_message_id) {
                    Some(metadata) => {
                        let current_time = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .expect("Clock may have gone backwards")
                            .as_millis() as u64;
                        let latest_milestone_arrival_timestamp =
                            metadata.arrival_timestamp();
                        if current_time - latest_milestone_arrival_timestamp > 5 * 60 * 60000 {
                            ret = false;
                        }
                    }
                    None => ret = false,
                }
            }
            None => ret = false,
        }

        ret
    };
    // TODO: get public key of coordinator from protocol config
    let coordinator_public_key = String::from("52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649");
    let latest_milestone_message_id = tangle.get_milestone_message_id(tangle.get_latest_milestone_index()).unwrap().to_string();
    let latest_milestone_index = *tangle.get_latest_milestone_index();
    let solid_milestone_message_id = tangle.get_milestone_message_id(tangle.get_latest_milestone_index()).unwrap().to_string();
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

pub async fn get_milestones<B: Backend>(tangle: ResHandle<MsTangle<B>>, milestone_index: u32) -> Result<impl Reply, Rejection> {
    match tangle.get_milestone_message_id(MilestoneIndex(milestone_index)) {
        Some(message_id) => {
            match tangle.get_metadata(&message_id) {
                Some(metadata) => {
                    let timestamp = metadata.arrival_timestamp();
                    Ok(warp::reply::json(&DataResponse::new(GetMilestonesResponseBody {
                        milestone_index,
                        message_id: message_id.to_string(),
                        timestamp
                    })))
                }
                None => Err(reject::reject())
            }
        }
        None => Err(reject::reject())
    }
}