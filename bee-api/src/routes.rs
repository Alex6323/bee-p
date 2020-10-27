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

use crate::types::InfoResponse;
use bee_common_ext::node::ResHandle;
use bee_protocol::tangle::MsTangle;
use bee_storage::storage::Backend;
use std::{
    convert::Infallible,
    time::{SystemTime, UNIX_EPOCH},
};
use warp::reply::json;
use bee_message::MessageId;

pub async fn info<B: Backend>(tangle: ResHandle<MsTangle<B>>) -> Result<warp::reply::Json, Infallible> {
    let name = String::from("Bee");
    let version = String::from(env!("CARGO_PKG_VERSION"));
    let is_healthy = {
        let mut ret = true;

        if !tangle.is_synced() {
            ret = false;
        }

        // TODO: check if number of peers != 0

        if *tangle.get_latest_milestone_index() == 0 {
            ret = false;
        }

        match tangle.get_milestone_message_id(tangle.get_latest_milestone_index()) {
            Some(milestone_message_id) => {
                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Clock may have gone backwards")
                    .as_millis() as u64;
                let latest_milestone_arrival_timestamp =
                    tangle.get_metadata(&milestone_message_id).unwrap().arrival_timestamp();

                if current_time - latest_milestone_arrival_timestamp > 5 * 60 * 60000 {
                    ret = false;
                }
            }
            None => ret = false,
        }

        ret
    };
    let coordinator_public_key = String::from("52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649"); // TODO: access protocol config

    //let latest_milestone_message_id = tangle.get_milestone_message_id(tangle.get_latest_milestone_index()).unwrap();
    let latest_milestone_index = tangle.get_latest_milestone_index();
    //let solid_milestone_message_id = tangle.get_milestone_message_id(tangle.get_latest_milestone_index()).unwrap();
    let solid_milestone_index = tangle.get_latest_milestone_index();
    let pruning_index = tangle.get_pruning_index();
    let features = Vec::new(); // TODO: check enabled features

    Ok(warp::reply::json(&InfoResponse {
        name,
        version,
        is_healthy,
        coordinator_public_key,
        latest_milestone_message_id: MessageId::from([0; 32]),
        latest_milestone_index,
        solid_milestone_message_id: MessageId::from([0; 32]),
        solid_milestone_index,
        pruning_index,
        features,
    }))
}
