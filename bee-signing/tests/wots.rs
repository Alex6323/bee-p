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

    use bee_crypto::ternary::Kerl;
    use bee_signing::ternary::{WotsError, WotsSecurityLevel, WotsSpongePrivateKeyGeneratorBuilder};

    #[test]
    fn wots_generator_missing_security_level() {
        match WotsSpongePrivateKeyGeneratorBuilder::<Kerl>::default().build() {
            Ok(_) => unreachable!(),
            Err(err) => assert_eq!(err, WotsError::MissingSecurityLevel),
        }
    }

    #[test]
    fn wots_generator_valid() {
        let security_levels = vec![
            WotsSecurityLevel::Low,
            WotsSecurityLevel::Medium,
            WotsSecurityLevel::High,
        ];
        for security in security_levels {
            assert_eq!(
                WotsSpongePrivateKeyGeneratorBuilder::<Kerl>::default()
                    .security_level(security)
                    .build()
                    .is_ok(),
                true
            );
        }
    }
}
