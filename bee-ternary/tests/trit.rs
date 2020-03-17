mod common;
use self::common::*;

use std::convert::TryFrom;
use bee_ternary::*;

#[test]
fn convert_correct() {
    assert_eq!(Utrit::from(0), Utrit::Zero);
    assert_eq!(Utrit::from(1), Utrit::One);
    assert_eq!(Utrit::from(2), Utrit::Two);

    assert_eq!(Into::<i8>::into(Utrit::Zero), 0i8);
    assert_eq!(Into::<i8>::into(Utrit::One), 1i8);
    assert_eq!(Into::<i8>::into(Utrit::Two), 2i8);
}

#[test]
fn convert_balanced() {
    assert_eq!(Utrit::from(Btrit::NegOne), Utrit::Zero);
    assert_eq!(Utrit::from(Btrit::Zero), Utrit::One);
    assert_eq!(Utrit::from(Btrit::PlusOne), Utrit::Two);
}

#[test]
#[should_panic]
fn convert_incorrect_0() { Utrit::from(-1); }

#[test]
#[should_panic]
fn convert_incorrect_1() { Utrit::from(3); }

#[test]
#[should_panic]
fn convert_incorrect_2() { Btrit::from(-2); }

#[test]
#[should_panic]
fn convert_incorrect_3() { Btrit::from(2); }

#[test]
fn tryte() {
    for c in &TRYTE_ALPHABET {
        println!("{}", c);
        let tryte = Tryte::try_from(*c).unwrap();
        assert_eq!(
            tryte.as_trits(),
            util::trytes_to_trits_buf(&format!("{}", c)).as_slice(),
        );
        assert_eq!(
            char::from(tryte),
            *c,
        );
    }
}
