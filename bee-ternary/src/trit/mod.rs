use std::convert::TryFrom;

pub mod balanced;
pub mod unbalanced;

// Reexports
pub use self::{
    balanced::Btrit,
    unbalanced::Utrit,
};

use std::fmt;

pub trait Trit: Copy + Sized + fmt::Debug + PartialEq + ShiftTernary + TryFrom<i8> {
    fn checked_increment(self) -> Option<Self>;

    fn zero() -> Self;
}

pub trait ShiftTernary: Sized {
    type Target: ShiftTernary<Target=Self>;

    fn shift(self) -> Self::Target;
}
