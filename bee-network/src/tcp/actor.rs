use crate::address::Address;
use crate::events::EventPublisher as EventPub;
use crate::shutdown::ShutdownListener;

pub struct TcpActor {
    binding_addr: Address,
    notifier: EventPub,
    publisher: EventPub,
    shutdown: ShutdownListener,
}

impl TcpActor {
    pub fn new(binding_addr: Address, notifier: EventPub, publisher: EventPub, shutdown: ShutdownListener) -> Self {
        Self {
            binding_addr,
            notifier,
            publisher,
            shutdown,
        }
    }

    pub async fn run(self) {
        //
    }
}
