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

use bee_storage::Backend;
use bee_common_ext::generic_static::{GenericStatic, GenericStaticGuard};
use async_std::task::block_on;

pub struct Storage {
    inner: Box<dyn Backend>,
}

static STORAGE: GenericStatic = GenericStatic::new();

pub fn deinit<B: Backend>() {
    let Storage { inner } = STORAGE.remove_inner().expect("Tried to deinit storage for a second time");
    block_on(Box::downcast::<B>(inner).unwrap().shutdown()).expect("Failed to shutdown storage");
}

pub fn tangle() -> GenericStaticGuard<'static, Storage> {
    STORAGE.get()
}


pub fn init<B: Backend>() {
    STORAGE.init_with(|| Storage {
        inner: Box::new(block_on(B::start(todo!()))
            .expect("Failed to start storage")),
    });
}
