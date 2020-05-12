// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::fmt;

#[derive(Clone)]
pub struct Utf8Message {
    data: String,
}

impl Utf8Message {
    pub fn new(s: &str) -> Self {
        Self { data: s.into() }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            data: String::from_utf8(bytes.to_vec()).unwrap(),
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        Vec::from(self.data.as_bytes())
    }
}

impl fmt::Display for Utf8Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.data)
    }
}
