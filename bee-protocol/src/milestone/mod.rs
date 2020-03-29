mod milestone;
mod validator;

pub use milestone::MilestoneIndex;
pub(crate) use validator::{
    MilestoneValidatorWorker,
    MilestoneValidatorWorkerEvent,
};
