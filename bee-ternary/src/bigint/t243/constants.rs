use lazy_static::lazy_static;

use crate::{
    Btrit,
    Utrit,
    bigint::{
        T243,
        U384,
        common::{
            LittleEndian,
            U32Repr,
        },
    },
};

lazy_static! {
    pub static ref BTRIT_ZERO: T243<Btrit> = T243::<Btrit>::zero();
    pub static ref BTRIT_ONE: T243<Btrit> = T243::<Btrit>::one();
    pub static ref BTRIT_NEG_ONE: T243<Btrit> = T243::<Btrit>::neg_one();

    pub static ref UTRIT_ZERO: T243<Utrit> = T243::<Utrit>::zero();
    pub static ref UTRIT_ONE: T243<Utrit> = T243::<Utrit>::one();
    pub static ref UTRIT_TWO: T243<Utrit> = T243::<Utrit>::two();

    pub static ref UTRIT_U384_MAX: T243<Utrit> = {
        From::from(U384::<LittleEndian, U32Repr>::max())
    };

    pub static ref UTRIT_U384_MAX_HALF: T243<Utrit> = {
        let mut u384_max = U384::<LittleEndian, U32Repr>::max();
        u384_max.divide_by_two();
        From::from(u384_max)
    };
}
