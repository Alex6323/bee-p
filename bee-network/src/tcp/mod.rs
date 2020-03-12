pub mod actor;
pub mod connection;

use crate::address::Address;
use crate::endpoint::EndpointId;
use crate::events::EventPublisher as Notifier;

pub async fn connect(id: &EndpointId, addr: &Address, notifier: Notifier) -> bool {
    // TODO
    unimplemented!()
}
