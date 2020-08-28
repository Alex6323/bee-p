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

// #![feature(test)]
// extern crate test;

use dashmap::DashMap;
use std::any::{Any, TypeId};

#[derive(Default)]
pub struct Bus<'a> {
    listeners: DashMap<TypeId, Vec<Box<dyn Fn(&dyn Any) + Send + Sync + 'a>>>,
}

impl<'a> Bus<'a> {
    pub fn dispatch<E: Any>(&self, event: E) {
        self.listeners
            .get_mut(&TypeId::of::<E>())
            .map(|mut ls| ls.iter_mut().for_each(|l| l(&event)));
    }

    pub fn add_listener<E: Any>(&self, handler: impl Fn(&E) + Send + Sync + 'a) {
        self.listeners
            .entry(TypeId::of::<E>())
            .or_default()
            .push(Box::new(move |event| {
                handler(&event.downcast_ref().expect("Invalid event"))
            }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use test::{Bencher, black_box};

    struct Foo;

    #[test]
    fn basic() {
        let bus = Bus::default();

        bus.add_listener(|_: &Foo| println!("Received a foo!"));

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
}
