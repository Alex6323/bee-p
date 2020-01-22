pub(crate) trait SplitInteger: Copy {
    type High;
    type Low;
    fn hi(self) -> Self::High;
    fn lo(self) -> Self::Low;
}

impl SplitInteger for i64 {
    type High = i32;
    type Low = u32;

    fn hi(self) -> Self::High {
        (self >> 32) as i32
    }

    fn lo(self) -> Self::Low {
        self as u32
    }
}

impl SplitInteger for u64 {
    type High = u32;
    type Low = u32;

    fn hi(self) -> Self::High {
        (self >> 32) as u32
    }

    fn lo(self) -> Self::Low {
        self as u32
    }
}

pub(crate) trait OverflowingAddExt<Rhs = Self> {
    type Output;
    fn overflowing_add_with_carry(self, other: Rhs, carry: Rhs) -> (Self::Output, bool);
}

impl OverflowingAddExt for u32 {
    type Output = Self;

    fn overflowing_add_with_carry(self, other: u32, carry: u32) -> (Self::Output, bool) {
        let (sum, first_overflow) = self.overflowing_add(other);
        let (sum, second_overflow) = sum.overflowing_add(carry);

        (sum, first_overflow | second_overflow)
    }
}


#[cfg(test)]
mod tests {
    use super::SplitInteger;

    #[test]
    fn split_i64_hi_minus_one() {
        assert_eq!((-1i64).hi(), -1i32);
    }

    #[test]
    fn split_i64_hi_min() {
        assert_eq!(i64::min_value().hi(), i32::min_value());
    }

    #[test]
    fn split_i64_lo_minus_one() {
        assert_eq!((-1i64).lo(), u32::max_value());
    }

    #[test]
    fn split_i64_lo_min() {
        assert_eq!(i64::min_value().lo(), 0u32);
    }
}
