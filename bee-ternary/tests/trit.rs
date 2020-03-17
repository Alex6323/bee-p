mod common;
use self::common::*;

use std::convert::TryFrom;
use bee_ternary::*;

#[test]
fn convert_correct() {
    assert_eq!(Utrit::from(-1), Utrit::NegOne);
    assert_eq!(Utrit::from(0), Utrit::Zero);
    assert_eq!(Utrit::from(1), Utrit::PlusOne);

    assert_eq!(Into::<i8>::into(Utrit::NegOne), -1i8);
    assert_eq!(Into::<i8>::into(Utrit::Zero), 0i8);
    assert_eq!(Into::<i8>::into(Utrit::PlusOne), 1i8);
}

#[test]
fn convert_balanced() {
    assert_eq!(Utrit::from(Btrit::NegOne), Utrit::NegOne);
    assert_eq!(Utrit::from(Btrit::Zero), Utrit::Zero);
    assert_eq!(Utrit::from(Btrit::PlusOne), Utrit::PlusOne);
}

#[test]
#[should_panic]
fn convert_incorrect_0() { Utrit::from(-2); }

#[test]
#[should_panic]
fn convert_incorrect_1() { Utrit::from(2); }

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
