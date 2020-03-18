use std::fmt;
use super::{Trit, Btrit, ToggleTernary};

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Utrit {
    Zero = 0,
    One = 1,
    Two = 2,
}

use Utrit::*;

impl From<i8> for Utrit {
    fn from(x: i8) -> Self {
        Self::try_convert(x as u8)
            .unwrap_or_else(|_| panic!("Invalid unbalanced trit representation '{}'", x))
    }
}

impl fmt::Display for Utrit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", *self as i8)
    }
}

impl From<Utrit> for i8 {
    fn from(value: Utrit) -> Self {
        value as i8
    }
}

impl From<Btrit> for Utrit {
    fn from(trit: Btrit) -> Self {
        // TODO: Fully review this
        unsafe { std::mem::transmute(trit as i8 + 1) }
    }
}

impl ToggleTernary for Utrit {
    type Target = Btrit;

    fn toggle(self) -> Self::Target {
        match self {
            Zero => Self::Target::NegOne,
            One => Self::Target::Zero,
            Two => Self::Target::PlusOne,
        }
    }
}

impl Trit for Utrit {
    type Repr = u8;

    fn try_convert(x: Self::Repr) -> Result<Self, ()> {
        match x {
            0 => Ok(Zero),
            1 => Ok(One),
            2 => Ok(Two),
            _ => Err(()),
        }
    }

    fn checked_increment(self) -> Option<Self> {
        match self {
            Zero => Some(One),
            One => Some(Two),
            Two => None,
        }
    }

    fn zero() -> Self {
        Self::Zero
    }
}

impl Utrit {
    pub(crate) fn from_u8(x: u8) -> Self {
        match x {
            0 => Zero,
            1 => One,
            2 => Two,
            x => panic!("Invalid trit representation '{}'", x),
        }
    }

    pub(crate) fn into_u8(self) -> u8 {
        match self {
            Zero => 0,
            One => 1,
            Two => 2,
        }
    }
}
