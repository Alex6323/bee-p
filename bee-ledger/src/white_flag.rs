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

use crate::{error::Error, metadata::WhiteFlagMetadata, spent::Spent, storage::Backend};

use bee_common_ext::node::{Node, ResHandle};
use bee_message::{
    payload::{transaction::Input, Payload},
    Message, MessageId,
};
use bee_protocol::tangle::MsTangle;
use bee_storage::access::Insert;

use std::{collections::HashSet, ops::Deref};

const IOTA_SUPPLY: u64 = 2_779_530_283_277_761;

#[inline]
fn on_message<N: Node>(
    tangle: &MsTangle<N::Backend>,
    storage: &ResHandle<N::Backend>,
    message_id: &MessageId,
    message: &Message,
    metadata: &mut WhiteFlagMetadata,
) -> Result<(), Error>
where
    N::Backend: Backend,
{
    let mut conflicting = false;

    metadata.num_messages_referenced += 1;

    if let Some(Payload::Transaction(transaction)) = message.payload() {
        // let transaction_id = transaction.id();
        let essence = transaction.essence();
        let inputs = essence.inputs();
        let outputs = essence.outputs();

        // TODO check transaction syntax here ?

        for input in inputs {
            if let Input::UTXO(utxo_input) = input {
                let output_id = utxo_input.output_id();

                if metadata.spent_outputs.contains_key(output_id) {
                    conflicting = true;
                    break;
                }
            } else {
                return Err(Error::UnsupportedInputType);
            };
        }

        // If not conflicting

        for output in outputs {}
    } else {
        metadata.num_messages_excluded_no_transaction += 1;
    }

    if conflicting {
        metadata.num_messages_excluded_conflicting += 1;
    } else {
        metadata.messages_included.push(*message_id);
    }

    // metadata
    //     .spent_outputs
    //     .insert(*output_id, Spent::new(transaction_id, metadata.index));

    tangle.update_metadata(message_id, |message_metadata| {
        message_metadata.flags_mut().set_conflicting(conflicting);
        message_metadata.set_milestone_index(metadata.index);
        // TODO pass actual ms timestamp
        message_metadata.confirm();
    });

    Ok(())
}

pub(crate) async fn visit_dfs<N: Node>(
    tangle: &MsTangle<N::Backend>,
    storage: &ResHandle<N::Backend>,
    root: MessageId,
    metadata: &mut WhiteFlagMetadata,
) -> Result<(), Error>
where
    N::Backend: Backend,
{
    let mut messages_ids = vec![root];
    let mut visited = HashSet::new();

    // TODO Tangle get message AND meta at the same time

    while let Some(message_id) = messages_ids.last() {
        let meta = match tangle.get_metadata(message_id) {
            Some(meta) => meta,
            None => {
                if !tangle.is_solid_entry_point(message_id) {
                    return Err(Error::MissingMessage(*message_id));
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
                    on_message::<N>(tangle, storage, message_id, &message, metadata)?;
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
                    return Err(Error::MissingMessage(*message_id));
                } else {
                    visited.insert(*message_id);
                    messages_ids.pop();
                }
            }
        }
    }

    Ok(())
}
