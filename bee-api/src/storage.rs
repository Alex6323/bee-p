use bee_message::{prelude::HashedIndex, MessageId};
use bee_storage::{access::Fetch, storage};

pub trait Backend: storage::Backend + Fetch<HashedIndex, Vec<MessageId>> {}

impl<T> Backend for T where T: storage::Backend + Fetch<HashedIndex, Vec<MessageId>> {}
