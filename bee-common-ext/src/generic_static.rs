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

//! A potentially static data structure that may be safely initiated, interpreted, and dropped as any type.

use std::{
    ops::Deref,
    marker::PhantomData,
    any::{Any, TypeId},
    mem::{MaybeUninit, ManuallyDrop},
};

pub struct GenericStatic {
    // TODO: Investigate using the `qcell` crate for this, it's probably faster... maybe?
    inner: spin::RwLock<GenericStaticInner>,
}

impl GenericStatic {
    pub const fn new() -> Self {
        Self {
            inner: spin::RwLock::new(GenericStaticInner {
                bytes: [0; BYTES],
                initiated: Err(false),
            }),
        }
    }

    pub fn init_with<T: Any>(&self, f: impl FnOnce() -> T) -> GenericStaticGuard<T> {
        let mut guard = self.inner.write();
        guard.init_with(f);
        GenericStaticGuard {
            guard: guard.downgrade(),
            phantom: PhantomData,
        }
    }

    pub fn get<T: Any>(&self) -> GenericStaticGuard<T> {
        GenericStaticGuard {
            guard: self.inner.read(),
            phantom: PhantomData,
        }
    }

    pub fn remove_inner<T: Any>(&self) -> Option<T> {
        self.inner.write().remove_inner::<T>()
    }
}

const BYTES: usize = 2048;

#[repr(align(64))]
pub struct GenericStaticInner {
    bytes: [u8; BYTES],
    initiated: Result<TypeId, bool>, // `true` means dropped, `false` means uninitiated
}

impl GenericStaticInner {
    fn init_with<T: Any>(&mut self, f: impl FnOnce() -> T) {
        match self.initiated {
            Err(false) => {
                let inner = unsafe { &mut *((&mut self.bytes as &mut [u8]).as_mut_ptr() as *mut MaybeUninit<ManuallyDrop<T>>) };

                // Test alignment
                if inner.as_ptr().align_offset(std::mem::align_of::<T>()) != 0 {
                    panic!("Alignment requirements are not satisfied for this generic static");
                }

                // TODO: Use `MaybeUninit::write` when stable, it's safe
                unsafe { inner.as_mut_ptr().write(ManuallyDrop::new(f())); }
                self.initiated = Ok(TypeId::of::<T>());
            },
            Err(true) => panic!("Attempted to reinit a dropped generic static"),
            Ok(ty) if ty != TypeId::of::<T>() => panic!("Attempted to reinit a generic static of differing type"),
            Ok(_) => {},
        }
    }

    fn get<T: Any>(&self) -> &T {
        match self.initiated {
            Err(false) => panic!("Attempted to get reference to uninitiated generic static"),
            Err(true) => panic!("Attempted to get reference to dropped generic static"),
            Ok(ty) if ty != TypeId::of::<T>() => panic!("Attempted to get reference to a generic static of differing type"),
            Ok(_) => {
                let inner = unsafe { &*((&self.bytes as &[u8]).as_ptr() as *const MaybeUninit<ManuallyDrop<T>>) };
                // TODO: Use `MaybeUninit::as_ref` when stable
                unsafe { (&*inner.as_ptr()).deref() }
            },
        }
    }

    fn remove_inner<T: Any>(&mut self) -> Option<T> {
        match self.initiated {
            Err(false) => panic!("Attempted to drop a generic static that has never been initiated in the first place"),
            Err(true) => None, // Already dropped
            Ok(ty) if ty != TypeId::of::<T>() => panic!("Attempted to drop a generic static of differing type"),
            Ok(_) => {
                let inner = unsafe { &mut *((&mut self.bytes as &mut [u8]).as_mut_ptr() as *mut MaybeUninit<ManuallyDrop<T>>) };
                let inner = unsafe { ManuallyDrop::take(&mut *inner.as_mut_ptr()) };
                self.initiated = Err(true);
                Some(inner)
            },
        }
    }
}

pub struct GenericStaticGuard<'a, T> {
    guard: spin::RwLockReadGuard<'a, GenericStaticInner>,
    phantom: PhantomData<T>,
}

impl<'a, T: Any> Deref for GenericStaticGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.guard.get()
    }
}

fn test_send<T: Send>() {}
fn test_send2<'a, T: Send + Sync>() { test_send::<GenericStaticGuard<'a, T>>() }
