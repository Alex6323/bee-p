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

#[macro_use]
extern crate criterion;

use criterion::Criterion;

fn bench_insert_transaction(_c: &mut Criterion) {
    todo!("insert a few thousand transactions as fast as possible")
}

criterion_group!(benches, bench_insert_transaction);
criterion_main!(benches);
