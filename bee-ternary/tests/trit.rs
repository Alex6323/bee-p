mod common;
use self::common::*;

use bee_ternary::*;

#[test]
fn convert_correct() {
    assert_eq!(UTrit::from(-1), UTrit::NegOne);
    assert_eq!(UTrit::from(0), UTrit::Zero);
    assert_eq!(UTrit::from(1), UTrit::PlusOne);

    assert_eq!(Into::<i8>::into(UTrit::NegOne), -1i8);
    assert_eq!(Into::<i8>::into(UTrit::Zero), 0i8);
    assert_eq!(Into::<i8>::into(UTrit::PlusOne), 1i8);
}

#[test]
fn convert_balanced() {
    assert_eq!(UTrit::from(BTrit::NegOne), UTrit::NegOne);
    assert_eq!(UTrit::from(BTrit::Zero), UTrit::Zero);
    assert_eq!(UTrit::from(BTrit::PlusOne), UTrit::PlusOne);
}

#[test]
#[should_panic]
fn convert_incorrect_0() { UTrit::from(-2); }

#[test]
#[should_panic]
fn convert_incorrect_1() { UTrit::from(2); }

