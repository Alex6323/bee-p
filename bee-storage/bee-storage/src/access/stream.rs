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

use crate::storage::Backend;

use futures::stream::Stream;

/// AsStream<'a, K, V> trait will extend the Backend with Scan operation for the key: K value: V collection
/// therefore it should be explicitly implemented for the corresponding Backend.
#[async_trait::async_trait]
pub trait AsStream<'a, K, V>: Backend {
    type Stream: Stream;
    /// This method returns the Stream object for the provided <K, V> collection in order to later execute async next()
    /// calls
    async fn stream(&'a self) -> Result<Self::Stream, Self::Error>
    where
        Self: Sized;
}
