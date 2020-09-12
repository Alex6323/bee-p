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

//! A crate that contains foundational building blocks for the IOTA Tangle.

pub mod batch;
pub mod delete;
pub mod fetch;
pub mod insert;

pub use batch::{ApplyBatch, Batch, BatchBuilder};
pub use delete::Delete;
pub use fetch::Fetch;
pub use insert::Insert;

pub trait Error: std::fmt::Debug {
    fn is_retryable(&self) -> bool;
    fn is_still_valid(&self) -> bool;
    fn error_msg(&self) -> Option<String>;
}
