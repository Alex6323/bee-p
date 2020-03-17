pub mod balanced;
pub mod unbalanced;

// Reexports
pub use self::{
    balanced::Btrit,
    unbalanced::Utrit,
};

use std::fmt;

pub trait Trit: Copy + Sized + fmt::Debug + PartialEq + ToggleTernary + From<Utrit> + From<Btrit> + Into<Utrit> + Into<Btrit> {
    type Repr: Copy;

    // TODO: Use std::convert::TryFrom
    fn try_convert(x: Self::Repr) -> Result<Self, ()>;
    fn checked_increment(self) -> Option<Self>;

    fn zero() -> Self;
}

pub trait ToggleTernary: Sized {
    type Target: ToggleTernary<Target=Self>;

    fn toggle(self) -> Self::Target;
}
