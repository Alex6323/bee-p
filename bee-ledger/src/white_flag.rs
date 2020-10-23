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

use crate::metadata::WhiteFlagMetadata;

use bee_message::{payload::Payload, Message, MessageId};
use bee_protocol::tangle::MsTangle;
use bee_storage::storage::Backend;

use std::collections::HashSet;

const IOTA_SUPPLY: u64 = 2_779_530_283_277_761;

#[derive(Debug)]
pub(crate) enum Error {
    MissingMessage,
}

#[inline]
fn on_message<B: Backend>(
    tangle: &MsTangle<B>,
    message_id: &MessageId,
    message: &Message,
    metadata: &mut WhiteFlagMetadata,
) {
    if let Payload::Transaction(transaction) = message.payload() {
    } else {
        metadata.num_messages_excluded_no_transaction += 1;
    }

    // let mut conflicting = false;
    // let (mutates, mutations) = bundle.ledger_mutations();
    //
    // if !mutates {
    //     metadata.num_tails_zero_value += 1;
    // } else {
    //     // First pass to look for conflicts.
    //     for (address, diff) in mutations.iter() {
    //         let balance = state.get_or_zero(&address) as i64 + diff;
    //
    //         if balance < 0 || balance.abs() as u64 > IOTA_SUPPLY {
    //             metadata.num_tails_conflicting += 1;
    //             conflicting = true;
    //             break;
    //         }
    //     }
    //
    //     if !conflicting {
    //         // Second pass to mutate the state.
    //         for (address, diff) in mutations {
    //             state.apply_single_diff(address.clone(), diff);
    //             metadata.diff.apply_single_diff(address, diff);
    //         }
    //
    //         metadata.tails_included.push(*message_id);
    //     }
    // }

    metadata.num_messages_referenced += 1;

    // // TODO this only actually confirm tails
    // tangle.update_metadata(&message_id, |meta| {
    //     meta.flags_mut().set_conflicting(conflicting);
    //     meta.confirm();
    //     meta.set_milestone_index(metadata.index);
    //     // TODO Set OTRSI, ...
    //     // TODO increment metrics confirmed, zero, value and conflict.
    // });
}

pub(crate) async fn visit_dfs<B: Backend>(
    tangle: &MsTangle<B>,
    root: MessageId,
    metadata: &mut WhiteFlagMetadata,
) -> Result<(), Error> {
    let mut messages_ids = vec![root];
    let mut visited = HashSet::new();

    // TODO Tangle get message AND meta at the same time

    while let Some(message_id) = messages_ids.last() {
        let meta = match tangle.get_metadata(message_id) {
            Some(meta) => meta,
            None => {
                if !tangle.is_solid_entry_point(message_id) {
                    return Err(Error::MissingMessage);
                } else {
                    visited.insert(*message_id);
                    messages_ids.pop();
                    continue;
                }
            }
        };

        if meta.flags().is_confirmed() {
            visited.insert(*message_id);
            messages_ids.pop();
            continue;
        }

        match tangle.get(message_id).await {
            Some(message) => {
                let parent1 = message.parent1();
                let parent2 = message.parent2();

                if visited.contains(parent1) && visited.contains(parent2) {
                    // TODO check valid and strict semantic
                    on_message(tangle, message_id, &message, metadata);
                    visited.insert(*message_id);
                    messages_ids.pop();
                } else if !visited.contains(parent1) {
                    messages_ids.push(*parent1);
                } else if !visited.contains(parent2) {
                    messages_ids.push(*parent2);
                }
            }
            None => {
                if !tangle.is_solid_entry_point(message_id) {
                    return Err(Error::MissingMessage);
                } else {
                    visited.insert(*message_id);
                    messages_ids.pop();
                }
            }
        }
    }

    Ok(())
}
