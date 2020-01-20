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

impl From<u8> for Trit {
    fn from(x: u8) -> Self {
        match x {
            0 => Trit::MinusOne,
            1 => Trit::Zero,
            2 => Trit::PlusOne,
            x => panic!("Invalid trit representation '{}'", x),
        }
    }
}

impl From<Trit> for i8 {
    fn from(t: Trit) -> Self {
        match t {
            Trit::MinusOne => -1,
            Trit::Zero => 0,
            Trit::PlusOne => 1,
        }
    }
}

impl From<Trit> for u8 {
    fn from(t: Trit) -> Self {
        match t {
            Trit::MinusOne => 0,
            Trit::Zero => 1,
            Trit::PlusOne => 2,
        }
    }
}

impl std::fmt::Debug for Trit {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Trit::MinusOne => write!(f, "{}", '-'),
            Trit::Zero => write!(f, "{}", '0'),
            Trit::PlusOne => write!(f, "{}", '1'),
        }
    }
}

#[cfg(test)]
mod should {
    use super::*;
    use std::convert::Into;

    #[test]
    fn create_trit_from_i8() {
        let trit = Trit::from(-1_i8);
        assert_eq!(Trit::MinusOne, trit);
        println!("{:?}", trit);

        let trit: Trit = 0_i8.into();
        assert_eq!(Trit::Zero, trit);
        println!("{:?}", trit);

        let trit: Trit = Trit::from(1_i8).into();
        assert_eq!(Trit::PlusOne, trit);
        println!("{:?}", trit);
    }

    #[test]
    fn create_i8_from_trit() {
        let trit = Trit::MinusOne;

        let int8: i8 = trit.into();

        println!("{:?}", int8);
    }

    #[test]
    fn create_trit_from_u8() {
        let trit = Trit::from(0_u8);
        assert_eq!(Trit::MinusOne, trit);
        println!("{:?}", trit);

        let trit: Trit = 1_u8.into();
        assert_eq!(Trit::Zero, trit);
        println!("{:?}", trit);

        let trit: Trit = Trit::from(2_u8).into();
        assert_eq!(Trit::PlusOne, trit);
        println!("{:?}", trit);
    }
}
