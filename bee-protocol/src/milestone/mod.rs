mod milestone;
mod validator;

pub use milestone::MilestoneIndex;
pub use validator::{
    MilestoneValidatorWorker,
    MilestoneValidatorWorkerEvent,
};
