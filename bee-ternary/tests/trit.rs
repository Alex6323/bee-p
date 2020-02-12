use ternary::*;

#[test]
fn convert_correct() {
    assert_eq!(Trit::from(-1), Trit::MinusOne);
    assert_eq!(Trit::from(0), Trit::Zero);
    assert_eq!(Trit::from(1), Trit::PlusOne);

    assert_eq!(Into::<i8>::into(Trit::MinusOne), -1i8);
    assert_eq!(Into::<i8>::into(Trit::Zero), 0i8);
    assert_eq!(Into::<i8>::into(Trit::PlusOne), 1i8);
}

#[test]
#[should_panic]
fn convert_incorrect_0() { Trit::from(-2); }

#[test]
#[should_panic]
fn convert_incorrect_1() { Trit::from(2); }

