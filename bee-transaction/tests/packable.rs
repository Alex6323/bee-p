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

#[cfg(test)]
mod tests {
    use bee_common_ext::packable::Packable;
    use bee_transaction::prelude::{Indexation, Message, MessageId, Payload};

    #[test]
    fn it_works() {
        let msg = Message::builder()
            .parent1(MessageId::new([
                0xF5, 0x32, 0xA5, 0x35, 0x45, 0x10, 0x32, 0x76, 0xB4, 0x68, 0x76, 0xC4, 0x73, 0x84, 0x6D, 0x98, 0x64,
                0x8E, 0xE4, 0x18, 0x46, 0x8B, 0xCE, 0x76, 0xDF, 0x48, 0x68, 0x64, 0x8D, 0xD7, 0x3E, 0x5D,
            ]))
            .parent2(MessageId::new([
                0x78, 0xD5, 0x46, 0xB4, 0x6A, 0xEC, 0x45, 0x57, 0x87, 0x21, 0x39, 0xA4, 0x8F, 0x66, 0xBC, 0x56, 0x76,
                0x87, 0xE8, 0x41, 0x35, 0x78, 0xA1, 0x43, 0x23, 0x54, 0x87, 0x32, 0x35, 0x89, 0x14, 0xA2,
            ]))
            .payload(Payload::Indexation(Box::new(Indexation::new(
                "0000".to_owned(),
                Box::new([0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x49, 0x6f, 0x74, 0x61]),
            ))))
            .build()
            .unwrap();

        let mut buf = vec![];

        msg.pack(&mut buf).unwrap();

        let msg_unpacked = Message::unpack(&mut buf.as_slice()).unwrap();

        assert_eq!(msg.parent1(), msg_unpacked.parent1());
        assert_eq!(msg.parent2(), msg_unpacked.parent2());
        // assert_eq!(msg.payload(), msg_unpacked.payload());
        // TODO check payload
    }
}
