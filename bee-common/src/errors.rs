#[derive(Debug)]
pub enum Errors {
    ConfigError {
        key: &'static str,
        msg: &'static str,
    },
    NetworkError,
    TransactionDeserializationError,
    TransactionBuilderError(&'static str),
}
