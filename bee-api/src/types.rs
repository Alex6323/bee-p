use serde::{Deserialize, Serialize};

/// Marker traits.
pub trait DataBody {}
pub trait ErrorBody {}

/// Data response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataResponse<T: DataBody> {
    pub data: T,
}

/// Error response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorResponse<T: ErrorBody> {
    pub error: T,
}

impl<T: DataBody> DataResponse<T> {
    /// Create a new data response.
    pub(crate) fn new(data: T) -> Self {
        Self { data }
    }
    /// Get data of the response.
    pub(crate) fn data(&self) -> &T {
        &self.data
    }
}

impl<T: ErrorBody> ErrorResponse<T> {
    /// Create a new error response.
    pub(crate) fn new(error: T) -> Self {
        Self { error }
    }
    /// Get error of the response.
    pub(crate) fn error(&self) -> &T {
        &self.error
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorResponseBody {
    pub code: String,
    pub message: String,
}

impl ErrorBody for ErrorResponseBody {}

/// Response body of GET /api/v1/info endpoint
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetInfoResponseBody {
    /// name of the node
    pub name: String,
    /// version of the node
    pub version: String,
    /// health status of the node
    #[serde(rename = "isHealthy")]
    pub is_healthy: bool,
    /// coordinator public key
    #[serde(rename = "coordinatorPublicKey")]
    pub coordinator_public_key: String,
    /// latest milestone message id
    #[serde(rename = "latestMilestoneMessageId")]
    pub latest_milestone_message_id: String,
    /// latest milestone index
    #[serde(rename = "latestMilestoneIndex")]
    pub latest_milestone_index: u32,
    /// latest milestone message id
    #[serde(rename = "solidMilestoneMessageId")]
    pub solid_milestone_message_id: String,
    /// solid milestone index
    #[serde(rename = "solidMilestoneIndex")]
    pub solid_milestone_index: u32,
    /// pruning index
    #[serde(rename = "pruningIndex")]
    pub pruning_index: u32,
    /// features
    pub features: Vec<String>,
}

impl DataBody for GetInfoResponseBody {}

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

/// Response body of GET /api/v1/tips endpoint
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetTipsResponseBody {
    /// index of the milestone
    #[serde(rename = "tip1MessageId")]
    pub tip_1_message_id: String,
    /// message id of the milestone
    #[serde(rename = "tip2MessageId")]
    pub tip_2_message_id: String,
}

impl DataBody for GetTipsResponseBody {}
