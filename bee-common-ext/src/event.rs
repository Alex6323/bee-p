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

type Listener<'a> = dyn Fn(&dyn Any) + Send + Sync + 'a;

#[derive(Default)]
pub struct Bus<'a> {
    listeners: DashMap<TypeId, Vec<(Box<Listener<'a>>, TypeId)>>,
}

impl<'a> Bus<'a> {
    pub fn dispatch<E: Any>(&self, event: E) {
        if let Some(mut ls) = self.listeners.get_mut(&TypeId::of::<E>()) {
            ls.iter_mut().for_each(|(l, _)| l(&event))
        }
    }

    pub fn add_listener<W: Any, E: Any, F: Fn(&E) + Send + Sync + 'a>(&self, handler: F) {
        self.listeners
            .entry(TypeId::of::<E>())
            .or_default()
            .push((Box::new(move |event| {
                handler(&event.downcast_ref().expect("Invalid event"))
            }), TypeId::of::<W>()));
    }

    pub fn purge_worker_listeners(&self, worker_id: TypeId) {
        self.listeners
            .iter_mut()
            .for_each(|mut listeners| listeners.retain(|(_, id)| *id != worker_id));
    }
}
