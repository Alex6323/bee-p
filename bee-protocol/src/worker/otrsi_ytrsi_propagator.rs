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

use crate::tangle::tangle;

use bee_common::{shutdown_stream::ShutdownStream, worker::Error as WorkerError};
use bee_common_ext::{node::Node, worker::Worker};
use bee_crypto::ternary::Hash;
use bee_tangle::helper::load_bundle_builder;

use async_trait::async_trait;
use futures::{
    channel::mpsc,
    stream::{Fuse, StreamExt},
};
use log::{info, warn};

use crate::{
    worker::tip_candidate_validator::{TipCandidateWorkerEvent, TipCandidateWorkerEvent::OtrsiYtrsiPropagated},
    Protocol,
};
use bee_transaction::Vertex;
use std::{
    cmp::{max, min},
    sync::Arc,
};

pub(crate) enum OtrsiYtrsiPropagatorWorkerEvent {
    Default(Hash),
    UpdateTransactionsReferencedByMilestone(Vec<Hash>),
}

pub(crate) struct OtrsiYtrsiPropagatorWorker {
    tip_candidate_validator: mpsc::UnboundedSender<TipCandidateWorkerEvent>,
}

#[async_trait]
impl<N: Node> Worker<N> for OtrsiYtrsiPropagatorWorker {
    type Config = ();
    type Error = WorkerError;
    type Event = OtrsiYtrsiPropagatorWorkerEvent;
    type Receiver = ShutdownStream<Fuse<mpsc::UnboundedReceiver<Self::Event>>>;

    async fn start(
        mut self,
        mut receiver: Self::Receiver,
        _node: Arc<N>,
        _config: Self::Config,
    ) -> Result<(), Self::Error> {
        info!("Running.");

        while let Some(event) = receiver.next().await {
            self.propagate(event);
        }

        info!("Stopped.");

        Ok(())
    }
}

impl OtrsiYtrsiPropagatorWorker {
    pub(crate) fn new(tip_candidate_validator: mpsc::UnboundedSender<TipCandidateWorkerEvent>) -> Self {
        Self {
            tip_candidate_validator,
        }
    }

    fn propagate(&mut self, event: OtrsiYtrsiPropagatorWorkerEvent) {
        let mut children = match &event {
            OtrsiYtrsiPropagatorWorkerEvent::Default(hash) => vec![*hash],
            OtrsiYtrsiPropagatorWorkerEvent::UpdateTransactionsReferencedByMilestone(hashes) => hashes.clone(),
        };
        while let Some(hash) = children.pop() {
            // get best otrsi and ytrsi from parents
            let tx = tangle().get(&hash).unwrap();
            let trunk_otsri = tangle().otrsi(tx.trunk());
            let branch_otsri = tangle().otrsi(tx.branch());
            let trunk_ytrsi = tangle().ytrsi(tx.trunk());
            let branch_ytrsi = tangle().ytrsi(tx.branch());

            if trunk_otsri.is_none() || branch_otsri.is_none() || trunk_ytrsi.is_none() || branch_ytrsi.is_none() {
                continue;
            }

            // check if already confirmed by update_transactions_referenced_by_milestone()
            if tangle().get_metadata(&hash).unwrap().cone_index().is_some() {
                continue;
            }

            // in case the transaction already inherited the best otrsi and ytrsi, continue
            let current_otrsi = tangle().get_metadata(&hash).unwrap().otrsi();
            let current_ytrsi = tangle().get_metadata(&hash).unwrap().ytrsi();
            let best_otrsi = max(trunk_otsri.unwrap(), branch_otsri.unwrap());
            let best_ytrsi = min(trunk_ytrsi.unwrap(), branch_ytrsi.unwrap());

            if current_otrsi.is_some()
                && current_ytrsi.is_some()
                && current_otrsi.unwrap() == best_otrsi
                && current_ytrsi.unwrap() == best_ytrsi
            {
                continue;
            }

            tangle().update_metadata(&hash, |metadata| {
                metadata.set_otrsi(best_otrsi);
                metadata.set_ytrsi(best_ytrsi);
            });

            // propagate otrsi and ytrsi to children
            for child in tangle().get_children(&hash) {
                children.push(child);
            }

            if let Err(e) = self
                .tip_candidate_validator
                .unbounded_send(TipCandidateWorkerEvent::OtrsiYtrsiPropagated(hash))
            {
                warn!("Failed to send hash to tip candidate validator: {:?}.", e);
            }
        }
        if let OtrsiYtrsiPropagatorWorkerEvent::UpdateTransactionsReferencedByMilestone(_) = event {
            tangle().update_tip_pool();
        }
    }
}
