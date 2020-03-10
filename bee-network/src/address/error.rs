use super::Address;

use err_derive::Error;

/// Errors that can happen when dealing with `Address`es.
#[derive(Debug, Error)]
pub enum AddressError {

    #[error(display = "error resolving domain name to address")]
    Io(#[source] std::io::Error),

    #[error(display = "error resolving domain name to address")]
    ResolveFailure,
}

pub type AddressResult = std::result::Result<Address, AddressError>;