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

use bee_api::{config::ApiConfigBuilder, rest};
use bee_ternary::{T1B1Buf, TritBuf};
use bee_transaction::{BundledTransaction, Hash};

fn main() {
    bee_tangle::init();

    let test_tx = BundledTransaction::from_trits(&TritBuf::<T1B1Buf>::zeros(BundledTransaction::trit_len())).unwrap();

    let mut rt = tokio::runtime::Runtime::new().expect("Error creating Tokio runtime");
    rt.block_on(bee_tangle::tangle().insert_transaction(test_tx, Hash::zeros()));

    assert_eq!(bee_tangle::tangle().contains_transaction(&Hash::zeros()), true);

    let socket_addr = ApiConfigBuilder::new().finish().rest_socket_addr();

    let mut rt = tokio::runtime::Runtime::new().expect("Error creating Tokio runtime");
    rt.block_on(rest::server::run(socket_addr));
}
