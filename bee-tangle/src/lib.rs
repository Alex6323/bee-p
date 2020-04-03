//! `bee-tangle`

#![deny(missing_docs)]

mod solidifier;
mod tangle;
mod vertex;

use solidifier::SolidifierState;
pub use tangle::Tangle;

use async_std::{
    sync::channel,
    task::spawn,
};

use bee_bundle::Hash;

use std::{
    ptr,
    sync::atomic::{
        AtomicPtr,
        Ordering,
    },
};

/// A temporary redefinition of `bee-bundle::Hash`.
pub type TransactionId = Hash;

/// A redefinition of a `usize`.
pub type MilestoneIndex = usize;

static TANGLE: AtomicPtr<Tangle> = AtomicPtr::new(ptr::null_mut());

const SOLIDIFIER_CHAN_CAPACITY: usize = 1000;

/// Initializes the Tangle singleton.
pub fn init() {
    let (sender, receiver) = channel::<Hash>(SOLIDIFIER_CHAN_CAPACITY);

    let solidifier_state = SolidifierState::new(receiver);

    TANGLE.store(Box::into_raw(Tangle::new(sender).into()), Ordering::Relaxed);

    spawn(solidifier::run(solidifier_state));
}

/// Returns the singleton instance of the Tangle.
pub fn tangle() -> &'static Tangle {
    let tangle = TANGLE.load(Ordering::Relaxed);
    if tangle.is_null() {
        panic!("Tangle cannot be null");
    } else {
        unsafe { &*tangle }
    }
}
