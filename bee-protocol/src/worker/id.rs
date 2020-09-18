#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub(crate) struct WorkerId(usize);

macro_rules! define_ids {
    ($($name:ident),+) => {
        #[repr(usize)]
        #[allow(non_camel_case_types)]
        enum WorkerIds {
            $($name,)+
        }

        impl WorkerId {
            $(pub(super) const fn $name() -> Self {
                Self(WorkerIds::$name as usize)
            })+
        }
    };
}

define_ids!(
    broadcaster,
    bundle_validator,
    milestone_validator,
    milestone_requester,
    transaction_requester,
    milestone_responder,
    transaction_responder,
    kickstart,
    milestone_solidifier,
    solid_propagator,
    status,
    tps,
    hasher,
    processor
);
