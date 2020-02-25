use std::{
    fmt,
    convert::TryFrom,
    marker::PhantomData,
};
use serde::{
    Serialize,
    Serializer,
    ser::SerializeSeq,
    Deserialize,
    Deserializer,
    de::{Visitor, SeqAccess, Error, Unexpected},
};
use crate::{
    Trit,
    Trits,
    TritBuf,
    RawEncoding,
    RawEncodingBuf,
};

// Serialisation

impl Serialize for Trit {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_i8((*self).into())
    }
}

impl<'a, T: RawEncoding> Serialize for &'a Trits<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for trit in self.iter() {
            seq.serialize_element(&trit)?;
        }
        seq.end()
    }
}

impl<T: RawEncodingBuf> Serialize for TritBuf<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for trit in self.iter() {
            seq.serialize_element(&trit)?;
        }
        seq.end()
    }
}

// Deserialisation

struct TritVisitor;

impl<'de> Visitor<'de> for TritVisitor {
    type Value = Trit;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a value between -1 and 1 inclusive")
    }

    fn visit_u64<E: Error>(self, trit: u64) -> Result<Self::Value, E> {
        i8::try_from(trit)
            .map_err(|_| ())
            .and_then(|trit| Trit::try_from(trit)
                .map_err(|_| ()))
            .map_err(|_| E::invalid_value(Unexpected::Unsigned(trit), &self))
    }

    fn visit_i64<E: Error>(self, trit: i64) -> Result<Self::Value, E> {
        i8::try_from(trit)
            .map_err(|_| ())
            .and_then(|trit| Trit::try_from(trit)
                .map_err(|_| ()))
            .map_err(|_| E::invalid_value(Unexpected::Signed(trit), &self))
    }

    fn visit_u8<E: Error>(self, trit: u8) -> Result<Self::Value, E> {
        i8::try_from(trit)
            .map_err(|_| ())
            .and_then(|trit| Trit::try_from(trit)
                .map_err(|_| ()))
            .map_err(|_| E::invalid_value(Unexpected::Unsigned(trit as u64), &self))
    }

    fn visit_i8<E: Error>(self, trit: i8) -> Result<Self::Value, E> {
        Trit::try_from(trit)
            .map_err(|_| E::invalid_value(Unexpected::Signed(trit as i64), &self))
    }
}

impl<'de> Deserialize<'de> for Trit {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_i8(TritVisitor)
    }
}

struct TritBufVisitor<T>(PhantomData<T>);

impl<'de, T: RawEncodingBuf> Visitor<'de> for TritBufVisitor<T> {
    type Value = TritBuf<T>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a sequence of trits")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut buf = TritBuf::with_capacity(seq.size_hint().unwrap_or(0));

        while let Some(trit) = seq.next_element()? {
            buf.push(trit);
        }

        Ok(buf)
    }
}

impl<'de, T: RawEncodingBuf> Deserialize<'de> for TritBuf<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_seq(TritBufVisitor::<T>(PhantomData))
    }
}
