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
