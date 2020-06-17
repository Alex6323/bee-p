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

    use bee_crypto::ternary::{CurlP27, CurlP81, Kerl, Sponge};
    use bee_signing::ternary::{
        PrivateKey, PrivateKeyGenerator, PublicKey, RecoverableSignature, Seed, TernarySeed, WotsSecurityLevel,
        WotsSpongePrivateKeyGeneratorBuilder,
    };
    use bee_ternary::{T1B1Buf, TryteBuf};

    const SEED: &str = "NNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNN";
    const MESSAGE: &str = "CHXHLHQLOPYP9NSUXTMWWABIBSBLUFXFRNWOZXJPVJPBCIDI99YBSCFYILCHPXHTSEYSYWIGQFERCRVDD";

    fn wots_sponge<S: Sponge + Default>() {
        let seed_trits = TryteBuf::try_from_str(SEED).unwrap().as_trits().encode::<T1B1Buf>();
        let message_trits = TryteBuf::try_from_str(MESSAGE).unwrap().as_trits().encode::<T1B1Buf>();
        let seed = TernarySeed::<S>::from_buf(seed_trits).unwrap();
        let security_levels = vec![
            WotsSecurityLevel::Low,
            WotsSecurityLevel::Medium,
            WotsSecurityLevel::High,
        ];
        for security in security_levels {
            for index in 0..5 {
                let private_key_generator = WotsSpongePrivateKeyGeneratorBuilder::<S>::default()
                    .security_level(security)
                    .build()
                    .unwrap();
                let mut private_key = private_key_generator.generate_from_seed(&seed, index).unwrap();
                let public_key = private_key.generate_public_key().unwrap();
                let signature = private_key.sign(message_trits.as_i8_slice()).unwrap();
                let recovered_public_key = signature.recover_public_key(message_trits.as_i8_slice()).unwrap();
                assert_eq!(public_key.as_bytes(), recovered_public_key.as_bytes());
                let valid = public_key.verify(message_trits.as_i8_slice(), &signature).unwrap();
                assert!(valid);
            }
        }
    }

    #[test]
    fn wots_kerl() {
        wots_sponge::<Kerl>();
    }

    #[test]
    fn wots_curl27() {
        wots_sponge::<CurlP27>();
    }

    #[test]
    fn wots_curl81() {
        wots_sponge::<CurlP81>();
    }
}
