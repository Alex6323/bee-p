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

use err_derive::Error;

/// Errors that can happen when dealing with `Address`es.
#[derive(Debug, Error)]
pub enum AddressError {
    #[error(display = "error resolving domain name to address")]
    Io(#[source] std::io::Error),

    #[error(display = "error parsing url")]
    UrlParseFailure,

    #[error(display = "error destructing url")]
    UrlDestructFailure,

    #[error(display = "unsupported protocol")]
    UnsupportedProtocol,

    #[error(display = "error resolving domain name to address")]
    ResolveFailure,
}

pub type AddressResult<T> = std::result::Result<T, AddressError>;
