use bee_ledger::{output::Output, spent::Spent};
use bee_message::{payload::transaction::OutputId, prelude::HashedIndex, MessageId};
use bee_storage::{access::Fetch, storage};

pub trait Backend:
    storage::Backend + Fetch<HashedIndex, Vec<MessageId>> + Fetch<OutputId, Output> + Fetch<OutputId, Spent>
{
}

impl<T> Backend for T where
    T: storage::Backend + Fetch<HashedIndex, Vec<MessageId>> + Fetch<OutputId, Output> + Fetch<OutputId, Spent>
{
}
