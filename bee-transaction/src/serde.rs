use crate::bundled::{
  Address, BundledTransactionField, Nonce, Payload, Tag, ADDRESS_TRIT_LEN, NONCE_TRIT_LEN, PAYLOAD_TRIT_LEN,
  TAG_TRIT_LEN,
};
use bee_ternary::{T1B1Buf, TritBuf, Trits, T1B1};
use serde::ser::Error as SerError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

macro_rules! impl_transaction_field_serde {
  ($field_name:ident, $trit_len:expr) => {
    impl Serialize for $field_name {
      fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
      where
        S: Serializer,
      {
        TritBuf::serialize(
          &Trits::<T1B1>::try_from_raw(self.to_inner().as_i8_slice(), $trit_len)
            .map_err(|_| SerError::custom("failed to get Trits from Hash"))?
            .to_buf::<T1B1Buf>(),
          serializer,
        )
      }
    }

    impl<'de> Deserialize<'de> for $field_name {
      fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
      where
        D: Deserializer<'de>,
      {
        TritBuf::deserialize(deserializer)
          .map(|buf: TritBuf<T1B1Buf>| $field_name::from_inner_unchecked(buf.as_slice().encode()))
      }
    }
  };
}

impl_transaction_field_serde!(Payload, PAYLOAD_TRIT_LEN);
impl_transaction_field_serde!(Address, ADDRESS_TRIT_LEN);
impl_transaction_field_serde!(Tag, TAG_TRIT_LEN);
impl_transaction_field_serde!(Nonce, NONCE_TRIT_LEN);

#[cfg(test)]
mod tests {
  use crate::bundled::{Address, Nonce, Payload, Tag};

  #[test]
  fn address_serde_check() {
    let address = Address::zeros();
    let serialized = serde_json::to_string(&address).expect("failed to serialize address");
    let deserialized: Address = serde_json::from_str(&serialized).expect("failed to deserialize address");

    assert!(address == deserialized);
  }

  #[test]
  fn nonce_serde_check() {
    let nonce = Nonce::zeros();
    let serialized = serde_json::to_string(&nonce).expect("failed to serialize nonce");
    let deserialized: Nonce = serde_json::from_str(&serialized).expect("failed to deserialize nonce");

    assert!(nonce == deserialized);
  }

  #[test]
  fn payload_serde_check() {
    let payload = Payload::zeros();
    let serialized = serde_json::to_string(&payload).expect("failed to serialize payload");
    let deserialized: Payload = serde_json::from_str(&serialized).expect("failed to deserialize payload");

    assert!(payload == deserialized);
  }

  #[test]
  fn tag_serde_check() {
    let tag = Tag::zeros();
    let serialized = serde_json::to_string(&tag).expect("failed to serialize tag");
    let deserialized: Tag = serde_json::from_str(&serialized).expect("failed to deserialize tag");

    assert!(tag == deserialized);
  }
}
