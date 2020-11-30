// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

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
