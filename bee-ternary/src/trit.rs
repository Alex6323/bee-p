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
        match x {
            -1 => Trit::MinusOne,
            0 => Trit::Zero,
            1 => Trit::PlusOne,
            x => panic!("Invalid trit representation '{}'", x),
        }
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
