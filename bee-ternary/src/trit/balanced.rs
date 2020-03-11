use std::fmt;
use super::{Trit, UTrit};

#[repr(i8)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BTrit {
    NegOne = -1,
    Zero = 0,
    PlusOne = 1,
}

impl From<i8> for BTrit {
    fn from(x: i8) -> Self {
        Self::try_from(x)
            .unwrap_or_else(|_| panic!("Invalid balanced trit representation '{}'", x))
    }
}

impl fmt::Debug for BTrit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BTrit::NegOne => write!(f, "{}", -1),
            BTrit::Zero => write!(f, "{}", 0),
            BTrit::PlusOne => write!(f, "{}", 1),
        }
    }
}

impl From<UTrit> for BTrit {
    fn from(trit: UTrit) -> Self {
        // TODO: Fully review this
        unsafe { std::mem::transmute(std::mem::transmute::<_, i8>(trit) - 1) }
    }
}

impl Into<i8> for BTrit {
    fn into(self) -> i8 {
        match self {
            BTrit::NegOne => -1,
            BTrit::Zero => 0,
            BTrit::PlusOne => 1,
        }
    }
}

impl Trit for BTrit {
    fn try_from(x: i8) -> Result<Self, ()> {
        match x {
            -1 => Ok(BTrit::NegOne),
            0 => Ok(BTrit::Zero),
            1 => Ok(BTrit::PlusOne),
            x => Err(()),
        }
    }

    fn checked_increment(self) -> Option<Self> {
        match self {
            BTrit::NegOne => Some(BTrit::Zero),
            BTrit::Zero => Some(BTrit::PlusOne),
            BTrit::PlusOne => None,
        }
    }
}
