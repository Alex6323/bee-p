// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

//! Type-length-value encoding/decoding.

use crate::message::{Header, Message, HEADER_SIZE};

#[derive(Debug)]
pub(crate) enum TlvError {
    InvalidAdvertisedType(u8, u8),
    InvalidAdvertisedLength(usize, usize),
    InvalidLength(usize),
}

/// Deserializes a TLV header and a byte buffer into a message.
///
/// # Arguments
///
/// * `header`  -   The TLV header to deserialize from.
/// * `bytes`   -   The byte buffer to deserialize from.
///
/// # Errors
///
/// * The advertised message type does not match the required message type.
/// * The advertised message length does not match the buffer length.
/// * The buffer length is not within the allowed size range of the required message type.
pub(crate) fn tlv_from_bytes<M: Message>(header: &Header, bytes: &[u8]) -> Result<M, TlvError> {
    if header.message_type != M::ID {
        return Err(TlvError::InvalidAdvertisedType(header.message_type, M::ID));
    }

    if header.message_length as usize != bytes.len() {
        return Err(TlvError::InvalidAdvertisedLength(
            header.message_length as usize,
            bytes.len(),
        ));
    }

    if !M::size_range().contains(&bytes.len()) {
        return Err(TlvError::InvalidLength(bytes.len()));
    }

    Ok(M::from_bytes(bytes))
}

/// Serializes a TLV header and a message into a byte buffer.
///
/// # Arguments
///
/// * `message` -   The message to serialize.
pub(crate) fn tlv_into_bytes<M: Message>(message: M) -> Vec<u8> {
    let size = message.size();
    let mut bytes = vec![0u8; HEADER_SIZE + size];
    let (header, payload) = bytes.split_at_mut(HEADER_SIZE);

    Header {
        message_type: M::ID,
        message_length: size as u16,
    }
    .to_bytes(header);
    message.into_bytes(payload);

    bytes
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::message::{
        v1::LegacyGossip, Handshake, Heartbeat, Message, MilestoneRequest, Transaction as TransactionMessage,
        TransactionRequest,
    };

    use bee_test::slices::slice_eq;

    use rand::Rng;

    use std::convert::TryInto;

    fn invalid_advertised_type<M: Message>() {
        match tlv_from_bytes::<M>(
            &Header {
                message_type: M::ID + 1,
                message_length: M::size_range().start as u16,
            },
            &Vec::with_capacity(M::size_range().start),
        ) {
            Err(TlvError::InvalidAdvertisedType(advertised_type, actual_type)) => {
                assert_eq!(advertised_type, M::ID + 1);
                assert_eq!(actual_type, M::ID);
            }
            _ => unreachable!(),
        }
    }

    fn invalid_advertised_length<M: Message>() {
        match tlv_from_bytes::<M>(
            &Header {
                message_type: M::ID,
                message_length: M::size_range().start as u16,
            },
            &vec![0u8; M::size_range().start + 1],
        ) {
            Err(TlvError::InvalidAdvertisedLength(advertised_length, actual_length)) => {
                assert_eq!(advertised_length, M::size_range().start);
                assert_eq!(actual_length, M::size_range().start + 1);
            }
            _ => unreachable!(),
        }
    }

    fn length_out_of_range<M: Message>() {
        match tlv_from_bytes::<M>(
            &Header {
                message_type: M::ID,
                message_length: M::size_range().start as u16 - 1,
            },
            &vec![0u8; M::size_range().start - 1],
        ) {
            Err(TlvError::InvalidLength(length)) => assert_eq!(length, M::size_range().start - 1),
            _ => unreachable!(),
        }

        match tlv_from_bytes::<M>(
            &Header {
                message_type: M::ID,
                message_length: M::size_range().end as u16,
            },
            &vec![0u8; M::size_range().end],
        ) {
            Err(TlvError::InvalidLength(length)) => assert_eq!(length, M::size_range().end),
            _ => unreachable!(),
        }
    }

    fn fuzz<M: Message>() {
        let mut rng = rand::thread_rng();

        for _ in 0..1000 {
            let length = rng.gen_range(M::size_range().start, M::size_range().end);
            let bytes_from: Vec<u8> = (0..length).map(|_| rand::random::<u8>()).collect();
            let message = tlv_from_bytes::<M>(
                &Header {
                    message_type: M::ID,
                    message_length: length as u16,
                },
                &bytes_from,
            )
            .unwrap();
            let bytes_to = tlv_into_bytes(message);

            assert_eq!(bytes_to[0], M::ID);
            assert_eq!(u16::from_be_bytes(bytes_to[1..3].try_into().unwrap()), length as u16);
            assert!(slice_eq(&bytes_from, &bytes_to[3..]));
        }
    }

    macro_rules! implement_tlv_tests {
        ($type:ty, $iat:tt, $ial:tt, $loor:tt, $fuzz:tt) => {
            #[test]
            fn $iat() {
                invalid_advertised_type::<$type>();
            }

            #[test]
            fn $ial() {
                invalid_advertised_length::<$type>();
            }

            #[test]
            fn $loor() {
                length_out_of_range::<$type>();
            }

            #[test]
            fn $fuzz() {
                fuzz::<$type>();
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
        TransactionMessage,
        invalid_advertised_type_transaction,
        invalid_advertised_length_transaction,
        length_out_of_range_transaction,
        fuzz_transaction
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
