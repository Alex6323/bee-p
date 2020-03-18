pub mod balanced;
pub mod unbalanced;

// Reexports
pub use self::{
    balanced::Btrit,
    unbalanced::Utrit,
};

use std::fmt;

pub trait Trit: Copy + Sized + fmt::Debug + PartialEq + ShiftTernary + From<i8> {
    type Repr: Copy;

    // TODO: Use std::convert::TryFrom
    fn try_convert(x: Self::Repr) -> Result<Self, ()>;
    fn checked_increment(self) -> Option<Self>;

    fn zero() -> Self;
}

pub trait ShiftTernary: Sized {
    type Target: ShiftTernary<Target=Self>;

    fn shift(self) -> Self::Target;
}
