mod milestone;
mod validator;

pub use milestone::{
    Milestone,
    MilestoneIndex,
};
pub(crate) use validator::{
    MilestoneValidatorWorker,
    MilestoneValidatorWorkerEvent,
};
