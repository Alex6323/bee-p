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
use internment::Intern;
use std::{any::Any, cell::Cell, sync::Arc};

pub trait Event: Any {
    fn name() -> &'static str
    where
        Self: Sized;

    fn get_interned() -> Intern<String>
    where
        Self: Sized,
    {
        thread_local! {
            pub static INTERNED_NAME: Cell<Option<Intern<String>>> = Cell::new(None);
        }

        INTERNED_NAME.with(|name| match name.get() {
            Some(intern) => intern,
            None => {
                let intern = Intern::new(Self::name().to_string());
                name.set(Some(intern));
                intern
            }
        })
    }
}

#[derive(Clone)]
struct InnerEvent(Arc<dyn Any>);

#[derive(Default)]
pub struct Bus<'a> {
    listeners: DashMap<Intern<String>, Vec<Box<dyn FnMut(InnerEvent) + 'a>>>,
}

impl<'a> Bus<'a> {
    pub fn dispatch<E: Event>(&self, event: E) {
        let inner_event = InnerEvent(Arc::new(event));
        self.listeners
            .get_mut(&E::get_interned())
            .map(|mut ls| ls.iter_mut().for_each(|l| l(inner_event.clone())));
    }

    pub fn add_listener<E: Event>(&self, mut handler: impl FnMut(&E) + Send + Sync + 'a) {
        self.listeners
            .entry(E::get_interned())
            .or_default()
            .push(Box::new(move |inner_event| {
                handler(&inner_event.0.downcast_ref().expect("Invalid event"))
            }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use test::{Bencher, black_box};

    struct Foo;
    impl Event for Foo {
        fn name() -> &'static str {
            "foo"
        }
    }

    struct Bar;
    impl Event for Bar {
        fn name() -> &'static str {
            "bar"
        }
    }

    #[test]
    fn basic() {
        let bus = Bus::default();

        bus.add_listener(|_: &Foo| println!("Received a foo!"));

        bus.dispatch(Foo);
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
