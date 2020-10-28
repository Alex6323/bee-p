use crate::dto::DataBody;
use serde::{Serialize, Deserialize};

/// Response body of GET /api/v1/info endpoint
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetMilestonesResponseBody {
    /// index of the milestone
    #[serde(rename = "milestoneIndex")]
    pub milestone_index: u32,
    /// message id of the milestone
    #[serde(rename = "messageId")]
    pub message_id: String,
    /// timestamp of the milestone
    pub timestamp: u64,
}

impl DataBody for GetMilestonesResponseBody {}