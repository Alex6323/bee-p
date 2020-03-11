use crate::commands::CommandReceiver as Commands;
use crate::events::EventSubscriber as Events;
use crate::events::EventPublisher as EventPub;
use crate::shutdown::ShutdownListener;

pub struct EndpointActor {
    commands: Commands,
    internals: Events,
    notifier: EventPub,
    publisher: EventPub,
    shutdown: ShutdownListener,
}

impl EndpointActor {
    pub fn new(
        commands: Commands,
        internals: Events,
        notifier: EventPub,
        publisher: EventPub,
        shutdown: ShutdownListener
    ) -> Self {

        Self {
            commands,
            internals,
            notifier,
            publisher,
            shutdown,
         }
    }

    pub async fn run(self) {
        //
    }
}