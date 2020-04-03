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

    let tangle = TANGLE.load(Ordering::Relaxed);
    if !tangle.is_null() {
        panic!("Already initialized");
    } else {
        TANGLE.store(Box::into_raw(Tangle::new(sender).into()), Ordering::Relaxed);
    }

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

/// Deallocates the Tangle singleton.
pub fn exit() {
    let tangle = TANGLE.swap(ptr::null_mut(), Ordering::Relaxed);
    if tangle.is_null() {
        return;
    } else {
        let _ = unsafe { Box::from_raw(tangle) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_and_exit() {
        init();
        let _ = tangle();
        exit();
    }

    #[test]
    #[should_panic]
    #[ignore]
    fn double_init_should_panic() {
        init();
        init();
        let _ = tangle();
        exit();
    }

    #[test]
    #[ignore]
    fn double_exit_should_not_double_free() {
        init();
        let _ = tangle();
        exit();
        exit();
    }
}
