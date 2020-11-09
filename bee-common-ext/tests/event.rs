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

use bee_common_ext::event::Bus;

struct Foo;

#[test]
fn basic() {
    let bus = Bus::default();

    bus.add_listener::<(), _, _>(|_: &Foo| println!("Received a foo!"));

    bus.dispatch(Foo);
}

#[test]
fn send_sync() {
    fn helper<T: Send + Sync>() {}
    helper::<Bus<'static>>();
}

// #[bench]
// fn bench_add_two(b: &mut Bencher) {
//     let bus = Bus::default();

//     bus.add_listener(|e: &Foo| { black_box(e); });

//     b.iter(|| {
//         bus.dispatch(Foo);
//     });
// }
