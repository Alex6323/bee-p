use std::fmt;
use super::{Trit, Btrit};

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Utrit {
    NegOne = 0,
    Zero = 1,
    PlusOne = 2,
}

impl From<i8> for Utrit {
    fn from(x: i8) -> Self {
        Self::try_from(x)
            .unwrap_or_else(|_| panic!("Invalid unbalanced trit representation '{}'", x))
    }
}

impl fmt::Display for Utrit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", *self as i8)
    }
}

impl From<Btrit> for Utrit {
    fn from(trit: Btrit) -> Self {
        // TODO: Fully review this
        unsafe { std::mem::transmute(trit as i8 + 1) }
    }
}

impl Into<i8> for Utrit {
    fn into(self) -> i8 {
        match self {
            Utrit::NegOne => -1,
            Utrit::Zero => 0,
            Utrit::PlusOne => 1,
        }
    }
}

impl Trit for Utrit {
    fn try_from(x: i8) -> Result<Self, ()> {
        match x {
            -1 => Ok(Utrit::NegOne),
            0 => Ok(Utrit::Zero),
            1 => Ok(Utrit::PlusOne),
            x => Err(()),
        }
    }

    fn checked_increment(self) -> Option<Self> {
        match self {
            Utrit::NegOne => Some(Utrit::Zero),
            Utrit::Zero => Some(Utrit::PlusOne),
            Utrit::PlusOne => None,
        }
    }
}

impl Utrit {
    pub(crate) fn from_u8(x: u8) -> Self {
        match x {
            0 => Utrit::NegOne,
            1 => Utrit::Zero,
            2 => Utrit::PlusOne,
            x => panic!("Invalid trit representation '{}'", x),
        }
    }

    pub(crate) unsafe fn from_u8_unchecked(x: u8) -> Self {
        // std::mem::transmute(x)
        Self::from_u8(x)
    }

    pub(crate) fn into_u8(self) -> u8 {
        match self {
            Utrit::NegOne => 0,
            Utrit::Zero => 1,
            Utrit::PlusOne => 2,
        }
    }
}
