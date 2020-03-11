use std::fmt;
use super::{Trit, BTrit};

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum UTrit {
    NegOne = 0,
    Zero = 1,
    PlusOne = 2,
}

impl From<i8> for UTrit {
    fn from(x: i8) -> Self {
        Self::try_from(x)
            .unwrap_or_else(|_| panic!("Invalid unbalanced trit representation '{}'", x))
    }
}

impl fmt::Debug for UTrit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UTrit::NegOne => write!(f, "{}", -1),
            UTrit::Zero => write!(f, "{}", 0),
            UTrit::PlusOne => write!(f, "{}", 1),
        }
    }
}

impl From<BTrit> for UTrit {
    fn from(trit: BTrit) -> Self {
        // TODO: Fully review this
        unsafe { std::mem::transmute(std::mem::transmute::<_, i8>(trit) + 1) }
    }
}

impl Into<i8> for UTrit {
    fn into(self) -> i8 {
        match self {
            UTrit::NegOne => -1,
            UTrit::Zero => 0,
            UTrit::PlusOne => 1,
        }
    }
}

impl Trit for UTrit {
    fn try_from(x: i8) -> Result<Self, ()> {
        match x {
            -1 => Ok(UTrit::NegOne),
            0 => Ok(UTrit::Zero),
            1 => Ok(UTrit::PlusOne),
            x => Err(()),
        }
    }

    fn checked_increment(self) -> Option<Self> {
        match self {
            UTrit::NegOne => Some(UTrit::Zero),
            UTrit::Zero => Some(UTrit::PlusOne),
            UTrit::PlusOne => None,
        }
    }
}

impl UTrit {
    pub(crate) fn from_u8(x: u8) -> Self {
        match x {
            0 => UTrit::NegOne,
            1 => UTrit::Zero,
            2 => UTrit::PlusOne,
            x => panic!("Invalid trit representation '{}'", x),
        }
    }

    pub(crate) unsafe fn from_u8_unchecked(x: u8) -> Self {
        // std::mem::transmute(x)
        Self::from_u8(x)
    }

    pub(crate) fn into_u8(self) -> u8 {
        match self {
            UTrit::NegOne => 0,
            UTrit::Zero => 1,
            UTrit::PlusOne => 2,
        }
    }
}
