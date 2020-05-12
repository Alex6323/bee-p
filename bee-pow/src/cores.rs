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

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Cores(pub(self) usize);

impl Cores {
    pub fn max() -> Self {
        Self(num_cpus::get())
    }
}

impl From<usize> for Cores {
    fn from(num_cores: usize) -> Self {
        let max_cores = num_cpus::get();
        if num_cores > max_cores {
            Self(max_cores)
        } else {
            Self(num_cores)
        }
    }
}

impl std::ops::Deref for Cores {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
