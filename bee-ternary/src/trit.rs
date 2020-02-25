use std::fmt;

#[repr(i8)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Trit {
    MinusOne = -1,
    Zero = 0,
    PlusOne = 1,
}

impl From<i8> for Trit {
    fn from(x: i8) -> Self {
        Self::try_from(x)
            .unwrap_or_else(|_| panic!("Invalid trit representation '{}'", x))
    }
}

impl fmt::Debug for Trit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Trit::MinusOne => write!(f, "{}", -1),
            Trit::Zero => write!(f, "{}", 0),
            Trit::PlusOne => write!(f, "{}", 1),
        }
    }
}

impl Into<i8> for Trit {
    fn into(self) -> i8 {
        match self {
            Trit::MinusOne => -1,
            Trit::Zero => 0,
            Trit::PlusOne => 1,
        }
    }
}

impl Trit {
    // TODO: Use std::convert::TryFrom
    pub fn try_from(x: i8) -> Result<Self, ()> {
        match x {
            -1 => Ok(Trit::MinusOne),
            0 => Ok(Trit::Zero),
            1 => Ok(Trit::PlusOne),
            x => Err(()),
        }
    }

    pub fn checked_increment(self) -> Option<Self> {
        match self {
            Trit::MinusOne => Some(Trit::Zero),
            Trit::Zero => Some(Trit::PlusOne),
            Trit::PlusOne => None,
        }
    }

    pub(crate) fn from_u8(x: u8) -> Self {
        match x {
            0 => Trit::MinusOne,
            1 => Trit::Zero,
            2 => Trit::PlusOne,
            x => panic!("Invalid trit representation '{}'", x),
        }
    }

    pub(crate) fn into_u8(self) -> u8 {
        match self {
            Trit::MinusOne => 0,
            Trit::Zero => 1,
            Trit::PlusOne => 2,
        }
    }
}
