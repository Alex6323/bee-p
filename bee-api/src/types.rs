use serde::{Deserialize, Serialize};

/// Marker trait for data bodies.
pub trait DataBody {}

/// Data response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataResponse<T: DataBody> {
    pub data: T,
}

impl<T: DataBody> DataResponse<T> {
    /// Create a new data response.
    pub(crate) fn new(data: T) -> Self {
        Self { data }
    }
    /// Get the body of the response.
    pub(crate) fn body(&self) -> &T {
        &self.data
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
}

/// Error response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

impl ErrorResponse {
    /// Create a new error response.
    pub(crate) fn new(error: ErrorBody) -> Self {
        Self { error }
    }
    /// Get the body of the response.
    pub(crate) fn body(&self) -> &ErrorBody {
        &self.error
    }
}

/// Response body of GET /api/v1/info
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

/// Response body of GET /api/v1/tips
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

/// Response body of GET /api/v1/milestone/{milestone_index}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetMilestoneByIndexResponseBody {
    /// index of the milestone
    #[serde(rename = "milestoneIndex")]
    pub milestone_index: u32,
    /// message id of the milestone
    #[serde(rename = "messageId")]
    pub message_id: String,
    /// timestamp of the milestone
    pub timestamp: u64,
}

impl DataBody for GetMilestoneByIndexResponseBody {}

/// Response body of GET /api/v1/messages/{message_id}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetMessageByIdResponseBody<T: Payload>(pub Message<T>);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message<T: Payload> {
    pub version: u32,
    #[serde(rename = "parent1MessageId")]
    pub parent_1_message_id: String,
    #[serde(rename = "parent2MessageId")]
    pub parent_2_message_id: String,
    pub payload: T,
    pub nonce: u64,
}

pub trait Payload {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IndexationPayload {
    #[serde(rename = "type")]
    pub kind: u32,
    pub index: String,
    pub data: String,
}
impl Payload for IndexationPayload {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MilestonePayload {
    #[serde(rename = "type")]
    pub kind: u32,
    pub index: u32,
    pub timestamp: u64,
    pub inclusion_merkle_proof: String,
    pub signature: String,
}
impl Payload for MilestonePayload {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionPayload<T: UnlockBlock> {
    #[serde(rename = "type")]
    pub kind: u32,
    pub essence: TransactionEssence,
    #[serde(rename = "unlockBlocks")]
    pub unlock_blocks: Vec<T>,
}
impl<T: UnlockBlock> Payload for TransactionPayload<T> {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionEssence {
    #[serde(rename = "type")]
    pub kind: u32,
    pub inputs: Vec<UtxoInput>,
    pub outputs: Vec<SigLockedSingleOutput>,
    pub payload: Option<IndexationPayload>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UtxoInput {
    #[serde(rename = "type")]
    pub kind: u32,
    #[serde(rename = "transactionId")]
    pub transaction_id: String,
    #[serde(rename = "transactionOutputIndex")]
    pub transaction_output_index: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SigLockedSingleOutput {
    #[serde(rename = "type")]
    pub kind: u32,
    pub address: Ed25519Address,
    pub amount: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ed25519Address {
    #[serde(rename = "type")]
    pub kind: u32,
    pub address: String,
}

pub trait UnlockBlock {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignatureUnlockBlock {
    #[serde(rename = "type")]
    pub kind: u32,
    pub signature: Ed25519Signature,
}

impl UnlockBlock for SignatureUnlockBlock {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ed25519Signature {
    #[serde(rename = "type")]
    pub kind: u32,
    #[serde(rename = "publicKey")]
    pub public_key: String,
    pub address: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReferenceUnlockBlock {
    #[serde(rename = "type")]
    pub kind: u32,
    pub reference: u32,
}

impl UnlockBlock for ReferenceUnlockBlock {}
