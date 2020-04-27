#[derive(Debug)]
pub enum Errors {
    NetworkError,
    TransactionDeserializationError,
    TransactionBuilderError(&'static str),
}
