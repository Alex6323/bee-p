macro_rules! def_and_impl_ternary {
    ($ident:ident, $len:expr) => {
        pub const LEN: usize = $len;

        #[derive(Clone, Debug)]
        pub struct $ident<T: Trit>(TritBuf<T1B1Buf<T>>);

        impl<T> $ident<T>
        where
            T: Trit,
        {
            pub fn from_trit_buf(trits_buf: TritBuf<T1B1Buf<T>>) -> Self {
                assert_eq!(trits_buf.len(), LEN);
                $ident(trits_buf)
            }

            /// Copies all elements from `Trits` into the inner `TritsBuf`.
            ///
            /// Panics if `Trits` does not have the same length as the inner buffer (243 elements).
            pub fn copy_from_trits<R>(&mut self, trits: &Trits<R>)
            where
                R: RawEncoding<Trit = T>,
            {
                assert_eq!(trits.len(), LEN);
                for (x, y) in self.0.iter_mut().zip(trits.trits()) {
                    *x = y;
                }
            }

            pub fn zero() -> Self {
                Self(TritBuf::zeros(LEN))
            }
            pub fn inner_ref(&self) -> &TritBuf<T1B1Buf<T>> {
                &self.0
            }

            pub fn inner_mut(&mut self) -> &mut TritBuf<T1B1Buf<T>> {
                &mut self.0
            }

            pub fn into_inner(self) -> TritBuf<T1B1Buf<T>> {
                self.0
            }
        }

        impl<T> $ident<T>
        where
            T: Trit,
            <T as ShiftTernary>::Target: Trit,
        {
            pub fn into_shifted(self) -> $ident<<T as ShiftTernary>::Target> {
                Self(self.0.into_shifted())
            }

        }

        impl $ident<Btrit> {
            pub fn one() -> Self {
                let mut t243 = Self::zero();
                t243.0.set(0, Btrit::PlusOne);
                t243
            }

            pub fn neg_one() -> Self {
                let mut t243 = Self::zero();
                t243.0.set(0, Btrit::NegOne);
                t243
            }

            pub fn two() -> Self {
                let mut t243 = Self::zero();
                t243.0.set(0, Btrit::NegOne);
                t243.0.set(1, Btrit::PlusOne);
                t243
            }

            pub fn neg_two() -> Self {
                let mut t243 = Self::zero();
                t243.0.set(0, Btrit::PlusOne);
                t243.0.set(1, Btrit::NegOne);
                t243
            }

            pub fn max() -> Self {
                Self(TritBuf::filled(LEN, Btrit::PlusOne))
            }

            pub fn min() -> Self {
                Self(TritBuf::filled(LEN, Btrit::NegOne))
            }
        }

        impl $ident<Utrit> {
            pub fn one() -> Self {
                let mut t243 = Self::zero();
                t243.0.set(0, Utrit::One);
                t243
            }

            pub fn two() -> Self {
                let mut t243 = Self::zero();
                t243.0.set(0, Utrit::Two);
                t243
            }

            pub fn half_max() -> Self {
                Self(TritBuf::filled(LEN, Utrit::One))
            }

            pub fn max() -> Self {
                Self(TritBuf::filled(LEN, Utrit::Two))
            }

            pub fn min() -> Self {
                Self::zero()
            }
        }

        impl<T: Trit> Default for $ident<T> {
            fn default() -> Self {
                Self::zero()
            }
        }

        impl<T: Trit> Eq for $ident<T> {}

        impl<T: Trit> Ord for $ident<T> {
            fn cmp(&self, other: &Self) -> Ordering {
                match self.partial_cmp(other) {
                    Some(ordering) => ordering,

                    // Cannot be reached because the order is total.
                    None => unreachable!(),
                }
            }
        }

        impl<T: Trit> PartialEq for $ident<T> {
            fn eq(&self, other: &Self) -> bool {
                self.0.eq(&other.0)
            }
        }

        impl<T: Trit> PartialOrd for $ident<T> {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                use Ordering::Equal;
                for (a, b) in self.0.trits().zip(other.0.trits()).rev() {
                     match a.cmp(&b) {
                         Equal => continue,
                         other_ordering => return Some(other_ordering),
                     }
                }
                Some(Equal)
            }
        }
    };
}

macro_rules! impl_const_functions {
    ( ( $($root:tt)* ), { $endianness:ty $(,)? }, { $repr:ty $(,)? } ) => {
        impl $($root)* < $endianness, $repr > {
            pub const fn from_array(inner: $repr) -> Self {
                Self {
                    inner,
                    _phantom: PhantomData,
                }
            }
        }
    };

    ( ( $($root:tt)* ), { $endianness:ty $(,)? }, { $repr:ty, $( $rest:ty ),+ $(,)? } ) => {

        impl_const_functions!( ( $($root)* ), { $endianness }, { $repr } );
        impl_const_functions!( ( $($root)* ), { $endianness }, { $( $rest ),+ } );
    };

    ( ( $($root:tt)* ), { $endianness:ty, $( $rest:ty ),+ $(,)? }, { $( $repr:ty ),+ $(,)? } ) => {

        impl_const_functions!( ( $($root)* ), { $endianness }, { $( $repr ),+ });

        impl_const_functions!( ( $($root)* ), { $( $rest ),+ }, { $( $repr ),+ });
    };
}

macro_rules! impl_constants {
    ( $( $t:ty => [ $( ( $fn:ident, $val:expr ) ),+ $(,)? ]),+ $(,)? ) => {
        $(
            impl $t {
                $(
                    pub const fn $fn() -> Self {
                        $val
                    }
                )+
            }
        )+
    };
}

macro_rules! impl_toggle_endianness {
    ( @inner
      ( $($root:tt)* ),
      $repr:ty,
      $src_endian:ty,
      $dst_endian:ty
    ) => {
        impl From< $($root)* < $src_endian, $repr >> for $($root)* <$dst_endian, $repr> {
            fn from(value: $($root)*<$src_endian, $repr>) -> Self {
                let mut inner = value.inner;
                inner.reverse();
                Self {
                    inner,
                    _phantom: PhantomData,
                }
            }
        }
    };

    ( ( $($root:tt)* ), $head:ty $(,)?) => {
        impl_toggle_endianness!(@inner ($($root)*), $head, LittleEndian, BigEndian);
        impl_toggle_endianness!(@inner ($($root)*), $head, BigEndian, LittleEndian);
    };

    ( ( $($root:tt)* ), $head:ty, $( $tail:ty ),+ $(,)?) => {
        impl_toggle_endianness!( ( $($root)* ), $head );
        impl_toggle_endianness!( ( $($root)* ), $( $tail ),+ );
    };
}
