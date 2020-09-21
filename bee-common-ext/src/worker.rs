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

use crate::node::Node;

use bee_common::worker::Error as WorkerError;

use async_trait::async_trait;
use futures::Stream;

use std::any::TypeId;

#[async_trait]
pub trait Worker<N: Node> {
    const DEPS: &'static [TypeId] = &[];

    type Event;
    type Receiver: Stream<Item = Self::Event>;

    async fn run(self, receiver: Self::Receiver) -> Result<(), WorkerError>;
}
