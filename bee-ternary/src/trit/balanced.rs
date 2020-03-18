use std::convert::TryFrom;
use std::fmt;
use super::{Trit, Utrit, ShiftTernary};

#[repr(i8)]
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Btrit {
    NegOne = -1,
    Zero = 0,
    PlusOne = 1,
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

impl ShiftTernary for Btrit {
    type Target = Utrit;

    fn shift(self) -> Self::Target {
        use Btrit::*;
        match self {
            NegOne => Self::Target::Zero,
            Zero => Self::Target::One,
            PlusOne => Self::Target::Two,
        }
    }
}

impl Trit for Btrit {
    fn checked_increment(self) -> Option<Self> {
        match self {
            Btrit::NegOne => Some(Btrit::Zero),
            Btrit::Zero => Some(Btrit::PlusOne),
            Btrit::PlusOne => None,
        }
    }

    fn zero() -> Self {
        Self::Zero
    }
}

impl TryFrom<i8> for Btrit {
    type Error = ();

    fn try_from(x: i8) -> Result<Self, Self::Error> {
        let converted = match x {
            -1 => Btrit::NegOne,
            0 => Btrit::Zero,
            1 => Btrit::PlusOne,
            _ => Err(())?,
        };
        Ok(converted)
    }
}

impl TryFrom<u8> for Btrit {
    type Error = ();

    fn try_from(x: u8) -> Result<Self, Self::Error> {
        let converted = match x {
            0 => Btrit::Zero,
            1 => Btrit::PlusOne,
            _ => Err(())?,
        };
        Ok(converted)
    }
}
