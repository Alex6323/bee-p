use std::fmt;
use super::{Trit, Utrit};

#[repr(i8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Btrit {
    NegOne = -1,
    Zero = 0,
    PlusOne = 1,
}

impl From<i8> for Btrit {
    fn from(x: i8) -> Self {
        Self::try_from(x)
            .unwrap_or_else(|_| panic!("Invalid balanced trit representation '{}'", x))
    }
}

impl fmt::Display for Btrit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", *self as i8)
    }
}

impl From<Btrit> for i8 {
    fn from(value: Btrit) -> Self {
        value as i8
    }
}

impl From<Utrit> for Btrit {
    fn from(trit: Utrit) -> Self {
        // TODO: Fully review this
        unsafe { std::mem::transmute(trit as i8 - 1) }
    }
}

impl Trit for Btrit {
    fn try_from(x: i8) -> Result<Self, ()> {
        match x {
            -1 => Ok(Btrit::NegOne),
            0 => Ok(Btrit::Zero),
            1 => Ok(Btrit::PlusOne),
            x => Err(()),
        }
    }

    fn checked_increment(self) -> Option<Self> {
        match self {
            Btrit::NegOne => Some(Btrit::Zero),
            Btrit::Zero => Some(Btrit::PlusOne),
            Btrit::PlusOne => None,
        }
    }
}
