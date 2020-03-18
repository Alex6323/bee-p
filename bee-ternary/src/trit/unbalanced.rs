use std::convert::TryFrom;
use std::fmt;
use super::{Trit, Btrit, ShiftTernary};

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Utrit {
    Zero = 0,
    One = 1,
    Two = 2,
}

use Utrit::*;

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

impl ShiftTernary for Utrit {
    type Target = Btrit;

    fn shift(self) -> Self::Target {
        match self {
            Zero => Self::Target::NegOne,
            One => Self::Target::Zero,
            Two => Self::Target::PlusOne,
        }
    }
}

impl Trit for Utrit {
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

impl TryFrom<i8> for Utrit {
    type Error = ();

    fn try_from(x: i8) -> Result<Self, Self::Error> {
        let converted = match x {
            0 => Zero,
            1 => One,
            2 => Two,
            _ => Err(())?,
        };
        Ok(converted)
    }
}

impl TryFrom<u8> for Utrit {
    type Error = ();

    fn try_from(x: u8) -> Result<Self, Self::Error> {
        let converted = match x {
            0 => Zero,
            1 => One,
            2 => Two,
            _ => Err(())?,
        };
        Ok(converted)
    }
}
