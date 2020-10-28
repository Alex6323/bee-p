use crate::dto::DataBody;
use serde::{Serialize, Deserialize};

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
    #[serde(rename = "solidMilestoneIndex", )]
    pub solid_milestone_index: u32,
    /// pruning index
    #[serde(rename = "pruningIndex")]
    pub pruning_index: u32,
    /// features
    pub features: Vec<String>,
}

impl DataBody for GetInfoResponseBody {}