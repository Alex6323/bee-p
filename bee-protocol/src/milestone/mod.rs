mod milestone;
mod validator;

pub(crate) use milestone::MilestoneIndex;
pub use validator::{
    MilestoneValidatorWorker,
    MilestoneValidatorWorkerEvent,
};
