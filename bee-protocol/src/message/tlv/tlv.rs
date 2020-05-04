//! Type-length-value encoding/decoding.

use crate::message::{Header, Message, MessageError, HEADER_SIZE};

/// Since the following methods have very common names, `from_bytes` and `into_bytes`, the sole purpose of this struct
/// is to give them a proper namespace to avoid confusion.
pub(crate) struct Tlv {}

impl Tlv {
    /// Deserializes a TLV header and a bytes buffer into a message.
    ///
    /// # Arguments
    ///
    /// * `header`  -   The TLV header to deserialize from.
    /// * `bytes`   -   The bytes buffer to deserialize from.
    ///
    /// # Errors
    ///
    /// * The advertised message type doesn't match the required message type.
    /// * The advertised message length doesn't match the buffer length.
    /// * The buffer length is not within the allowed size range of the required message type.
    pub(crate) fn from_bytes<M: Message>(header: &Header, bytes: &[u8]) -> Result<M, MessageError> {
        if header.message_type != M::ID {
            return Err(MessageError::InvalidAdvertisedType(header.message_type, M::ID));
        }

        if header.message_length as usize != bytes.len() {
            return Err(MessageError::InvalidAdvertisedLength(
                header.message_length as usize,
                bytes.len(),
            ));
        }

        if !M::size_range().contains(&bytes.len()) {
            return Err(MessageError::InvalidLength(bytes.len()));
        }

        Ok(M::from_bytes(bytes))
    }

    /// Serializes a TLV header and a message into a bytes buffer.
    ///
    /// # Arguments
    ///
    /// * `message` -   The message to serialize.
    pub(crate) fn into_bytes<M: Message>(message: M) -> Vec<u8> {
        let size = message.size();
        let mut bytes = vec![0u8; HEADER_SIZE + size];
        let (header, payload) = bytes.split_at_mut(HEADER_SIZE);

        header[0] = M::ID;
        header[1..].copy_from_slice(&(size as u16).to_be_bytes());
        message.into_bytes(payload);

        bytes
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::message::{
        v1::LegacyGossip, Handshake, Heartbeat, Message, MilestoneRequest, TransactionBroadcast, TransactionRequest,
    };

    use bee_test::slices::slice_eq;

    use rand::Rng;

    use std::convert::TryInto;

    fn invalid_advertised_type_generic<M: Message>() {
        match Tlv::from_bytes::<M>(
            &Header {
                message_type: M::ID + 1,
                message_length: M::size_range().start as u16,
            },
            &Vec::with_capacity(M::size_range().start),
        ) {
            Err(MessageError::InvalidAdvertisedType(advertised_type, actual_type)) => {
                assert_eq!(advertised_type, M::ID + 1);
                assert_eq!(actual_type, M::ID);
            }
            _ => unreachable!(),
        }
    }

    fn invalid_advertised_length_generic<M: Message>() {
        match Tlv::from_bytes::<M>(
            &Header {
                message_type: M::ID,
                message_length: M::size_range().start as u16,
            },
            &vec![0u8; M::size_range().start + 1],
        ) {
            Err(MessageError::InvalidAdvertisedLength(advertised_length, actual_length)) => {
                assert_eq!(advertised_length, M::size_range().start);
                assert_eq!(actual_length, M::size_range().start + 1);
            }
            _ => unreachable!(),
        }
    }

    fn length_out_of_range_generic<M: Message>() {
        match Tlv::from_bytes::<M>(
            &Header {
                message_type: M::ID,
                message_length: M::size_range().start as u16 - 1,
            },
            &vec![0u8; M::size_range().start - 1],
        ) {
            Err(MessageError::InvalidLength(length)) => assert_eq!(length, M::size_range().start - 1),
            _ => unreachable!(),
        }

        match Tlv::from_bytes::<M>(
            &Header {
                message_type: M::ID,
                message_length: M::size_range().end as u16,
            },
            &vec![0u8; M::size_range().end],
        ) {
            Err(MessageError::InvalidLength(length)) => assert_eq!(length, M::size_range().end),
            _ => unreachable!(),
        }
    }

    fn fuzz_generic<M: Message>() {
        let mut rng = rand::thread_rng();

        for _ in 0..1000 {
            let length = rng.gen_range(M::size_range().start, M::size_range().end);
            let bytes_from: Vec<u8> = (0..length).map(|_| rand::random::<u8>()).collect();
            let message = Tlv::from_bytes::<M>(
                &Header {
                    message_type: M::ID,
                    message_length: length as u16,
                },
                &bytes_from,
            )
            .unwrap();
            let bytes_to = Tlv::into_bytes(message);

            assert_eq!(bytes_to[0], M::ID);
            assert_eq!(u16::from_be_bytes(bytes_to[1..3].try_into().unwrap()), length as u16);
            assert!(slice_eq(&bytes_from, &bytes_to[3..]));
        }
    }

    macro_rules! implement_tlv_tests {
        ($type:ty, $iat:tt, $ial:tt, $loor:tt, $fuzz:tt) => {
            #[test]
            fn $iat() {
                invalid_advertised_type_generic::<$type>();
            }

            #[test]
            fn $ial() {
                invalid_advertised_length_generic::<$type>();
            }

            #[test]
            fn $loor() {
                length_out_of_range_generic::<$type>();
            }

            #[test]
            fn $fuzz() {
                fuzz_generic::<$type>();
            }
        };
    }

    implement_tlv_tests!(
        Handshake,
        invalid_advertised_type_handshake,
        invalid_advertised_length_handshake,
        length_out_of_range_handshake,
        fuzz_handshake
    );

    implement_tlv_tests!(
        LegacyGossip,
        invalid_advertised_type_legacy_gossip,
        invalid_advertised_length_legacy_gossip,
        length_out_of_range_legacy_gossip,
        fuzz_legacy_gossip
    );

    implement_tlv_tests!(
        MilestoneRequest,
        invalid_advertised_type_milestone_request,
        invalid_advertised_length_milestone_request,
        length_out_of_range_milestone_request,
        fuzz_milestone_request
    );

    implement_tlv_tests!(
        TransactionBroadcast,
        invalid_advertised_type_transaction_broadcast,
        invalid_advertised_length_transaction_broadcast,
        length_out_of_range_transaction_broadcast,
        fuzz_transaction_broadcast
    );

    implement_tlv_tests!(
        TransactionRequest,
        invalid_advertised_type_transaction_request,
        invalid_advertised_length_transaction_request,
        length_out_of_range_transaction_request,
        fuzz_transaction_request
    );

    implement_tlv_tests!(
        Heartbeat,
        invalid_advertised_type_heartbeat,
        invalid_advertised_length_heartbeat,
        length_out_of_range_heartbeat,
        fuzz_range_heartbeat
    );
}
