pub mod balanced;
pub mod unbalanced;

// Reexports
pub use self::{
    balanced::BTrit,
    unbalanced::UTrit,
};

use std::fmt;

pub trait Trit: Copy + Sized + fmt::Debug + From<UTrit> + Into<UTrit> + Into<i8> {
    // TODO: Use std::convert::TryFrom
    fn try_from(x: i8) -> Result<Self, ()>;
    fn checked_increment(self) -> Option<Self>;
}
