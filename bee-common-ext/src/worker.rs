// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::node::Node;

use async_trait::async_trait;

use std::any::{Any, TypeId};

#[async_trait]
pub trait Worker<N: Node>: Any + Send + Sync + Sized {
    type Config;
    type Error: std::error::Error;

    // TODO Replace with associated constant when stabilized.
    fn dependencies() -> &'static [TypeId] {
        &[]
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error>
    where
        Self: Sized;

    async fn stop(self, _node: &mut N) -> Result<(), Self::Error>
    where
        Self: Sized,
    {
        Ok(())
    }
}
