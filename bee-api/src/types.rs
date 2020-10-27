use bee_message::MessageId;
use bee_protocol::MilestoneIndex;
use hex::{FromHex, ToHex};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Marker traits.
pub trait DataBody {}
pub trait ErrorBody {}

/// Data response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DataResponse<T: DataBody> {
    data: T,
}

/// Error response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorResponse<T: ErrorBody> {
    error: T,
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
    /// Create a new data response.
    pub(crate) fn new(error: T) -> Self {
        Self { error }
    }
    /// Get data of the response.
    pub(crate) fn error(&self) -> &T {
        &self.error
    }
}

/// Data response of GET /api/v1/info endpoint
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetInfoBody {
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
    #[serde(
        rename = "latestMilestoneMessageId",
        serialize_with = "message_id_to_hex",
        deserialize_with = "hex_to_message_id"
    )]
    pub latest_milestone_message_id: MessageId,
    /// latest milestone index
    #[serde(
        rename = "latestMilestoneIndex",
        serialize_with = "milestone_index_to_u32",
        deserialize_with = "u32_to_milestone_index"
    )]
    pub latest_milestone_index: MilestoneIndex,
    /// latest milestone message id
    #[serde(
        rename = "solidMilestoneMessageId",
        serialize_with = "message_id_to_hex",
        deserialize_with = "hex_to_message_id"
    )]
    pub solid_milestone_message_id: MessageId,
    /// solid milestone index
    #[serde(
        rename = "solidMilestoneIndex",
        serialize_with = "milestone_index_to_u32",
        deserialize_with = "u32_to_milestone_index"
    )]
    pub solid_milestone_index: MilestoneIndex,
    /// pruning index
    #[serde(
        rename = "pruningIndex",
        serialize_with = "milestone_index_to_u32",
        deserialize_with = "u32_to_milestone_index"
    )]
    pub pruning_index: MilestoneIndex,
    /// features
    pub features: Vec<String>,
}

impl DataBody for GetInfoBody {}

fn message_id_to_hex<S>(message: &MessageId, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&hex::encode(message))
}

fn hex_to_message_id<'de, D>(d: D) -> Result<MessageId, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let mut bytes = [0u8; 32];
    String::deserialize(d)
        .and_then(|string| hex::decode_to_slice(string, &mut bytes).map_err(|err| Error::custom(err.to_string())));
    Ok(MessageId::new(bytes))
}

fn milestone_index_to_u32<S>(milestone_index: &MilestoneIndex, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_u32(**milestone_index)
}

fn u32_to_milestone_index<'de, D>(d: D) -> Result<MilestoneIndex, D::Error>
where
    D: Deserializer<'de>,
{
    u32::deserialize(d).and_then(|index| Ok(MilestoneIndex(index)))
}
