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

use async_trait::async_trait;

use std::any::{Any, TypeId};

#[async_trait]
pub trait Worker<N: Node>: Any + Send + Sync {
    type Config;
    type Error: std::error::Error;

    // TODO Replace with associated constant when stabilized.
    fn dependencies() -> &'static [TypeId] {
        &[]
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error>
    where
        Self: Sized;

    async fn stop(self) -> Result<(), Self::Error>
    where
        Self: Sized,
    {
        Ok(())
    }
}
