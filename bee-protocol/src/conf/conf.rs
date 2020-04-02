const CONF_COO_PUBLIC_KEY: (&str, &str) = (
    "protocol.coordinator.publicKey",
    "EQSAUZXULTTYZCLNJNTXQTQHOMOFZERHTCGTXOLTVAHKSA9OGAZDEKECURBRIXIJWNPFCQIOVFVVXJVD9",
);
const CONF_COO_SPONGE_TYPE: (&str, &str) = ("protocol.coordinator.sponge", "kerl");
const CONF_COO_SECURITY: (&str, u8) = ("protocol.coordinator.securityLevel", 2);
const CONF_COO_DEPTH: (&str, u8) = ("protocol.coordinator.depth", 23);
const CONF_MWM: (&str, u8) = ("protocol.mwm", 14);
const CONF_MILESTONE_REQUEST_SEND_WORKER_BOUND: (&str, u32) =
    ("protocol.channels.milestoneRequestSendWorkerBound", 1000);
const CONF_TRANSACTION_BROADCAST_SEND_WORKER_BOUND: (&str, u32) =
    ("protocol.channels.transactionBroadcastSendWorkerBound", 1000);
const CONF_TRANSACTION_REQUEST_SEND_WORKER_BOUND: (&str, u32) =
    ("protocol.channels.transactionRequestSendWorkerBound", 1000);
const CONF_HEARTBEAT_SEND_WORKER_BOUND: (&str, u32) = ("protocol.channels.heartbeatSendWorkerBound", 1000);
const CONF_MILESTONE_VALIDATOR_WORKER_BOUND: (&str, u32) = ("protocol.channels.milestoneValidatorWorkerBound", 1000);
const CONF_TRANSACTION_WORKER_BOUND: (&str, u32) = ("protocol.channels.transactionWorkerBound", 1000);
const CONF_TRANSACTION_RESPONDER_WORKER_BOUND: (&str, u32) =
    ("protocol.channels.transactionResponderWorkerBound", 1000);
const CONF_MILESTONE_RESPONDER_WORKER_BOUND: (&str, u32) = ("protocol.channels.milestoneResponderWorkerBound", 1000);
const CONF_TRANSACTION_REQUESTER_WORKER_BOUND: (&str, u32) =
    ("protocol.channels.transactionRequesterWorkerBound", 1000);
const CONF_MILESTONE_REQUESTER_WORKER_BOUND: (&str, u32) = ("protocol.channels.milestoneRequesterWorkerBound", 1000);
const CONF_RECEIVER_WORKER_BOUND: (&str, u32) = ("protocol.channels.receiverWorkerBound", 1000);

// TODO remove this one
pub(crate) const COORDINATOR_BYTES: [u8; 49] = [
    234, 56, 202, 174, 238, 197, 195, 253, 109, 14, 137, 227, 44, 144, 151, 188, 192, 45, 220, 236, 64, 168, 220, 197,
    22, 199, 188, 1, 45, 11, 107, 190, 49, 84, 147, 176, 184, 108, 223, 189, 17, 167, 184, 240, 213, 170, 111, 34, 0,
];
pub(crate) const COORDINATOR_SECURITY_LEVEL: u8 = 2;
pub(crate) const COORDINATOR_DEPTH: u8 = 23;
pub(crate) const MINIMUM_WEIGHT_MAGNITUDE: u8 = 14;

pub(crate) const MILESTONE_REQUEST_SEND_BOUND: usize = 1000;
pub(crate) const TRANSACTION_BROADCAST_SEND_BOUND: usize = 1000;
pub(crate) const TRANSACTION_REQUEST_SEND_BOUND: usize = 1000;
pub(crate) const HEARTBEAT_SEND_BOUND: usize = 1000;

#[derive(Default)]
pub struct ProtocolConfBuilder {
    coo_public_key: Option<String>,
    // TODO sponge type type ?
    coo_sponge_type: Option<String>,
    coo_security_level: Option<u8>,
    coo_depth: Option<u8>,
    mwm: Option<u8>,
    milestone_request_send_worker_bound: Option<u32>,
    transaction_broadcast_send_worker_bound: Option<u32>,
    transaction_request_send_worker_bound: Option<u32>,
    heartbeat_send_worker_bound: Option<u32>,
    milestone_validator_worker_bound: Option<u32>,
    transaction_worker_bound: Option<u32>,
    transaction_responder_worker_bound: Option<u32>,
    milestone_responder_worker_bound: Option<u32>,
    transaction_requester_worker_bound: Option<u32>,
    milestone_requester_worker_bound: Option<u32>,
    receiver_worker_bound: Option<u32>,
}

impl ProtocolConfBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_file(self) -> Self {
        // TODO load all fields
        self
    }

    pub fn coo_public_key(mut self, coo_public_key: String) -> Self {
        self.coo_public_key.replace(coo_public_key);
        self
    }

    pub fn coo_sponge_type(mut self, coo_sponge_type: String) -> Self {
        self.coo_sponge_type.replace(coo_sponge_type);
        self
    }

    pub fn coo_security_level(mut self, coo_security_level: u8) -> Self {
        self.coo_security_level.replace(coo_security_level);
        self
    }

    pub fn coo_depth(mut self, coo_depth: u8) -> Self {
        self.coo_depth.replace(coo_depth);
        self
    }

    pub fn mwm(mut self, mwm: u8) -> Self {
        self.mwm.replace(mwm);
        self
    }

    pub fn milestone_request_send_worker_bound(mut self, milestone_request_send_worker_bound: u32) -> Self {
        self.transaction_broadcast_send_worker_bound
            .replace(milestone_request_send_worker_bound);
        self
    }

    pub fn transaction_broadcast_send_worker_bound(mut self, transaction_broadcast_send_worker_bound: u32) -> Self {
        self.transaction_broadcast_send_worker_bound
            .replace(transaction_broadcast_send_worker_bound);
        self
    }

    pub fn transaction_request_send_worker_bound(mut self, transaction_request_send_worker_bound: u32) -> Self {
        self.transaction_request_send_worker_bound
            .replace(transaction_request_send_worker_bound);
        self
    }

    pub fn heartbeat_send_worker_bound(mut self, heartbeat_send_worker_bound: u32) -> Self {
        self.heartbeat_send_worker_bound.replace(heartbeat_send_worker_bound);
        self
    }

    pub fn milestone_validator_worker_bound(mut self, milestone_validator_worker_bound: u32) -> Self {
        self.milestone_validator_worker_bound
            .replace(milestone_validator_worker_bound);
        self
    }

    pub fn transaction_worker_bound(mut self, transaction_worker_bound: u32) -> Self {
        self.transaction_worker_bound.replace(transaction_worker_bound);
        self
    }

    pub fn transaction_responder_worker_bound(mut self, transaction_responder_worker_bound: u32) -> Self {
        self.transaction_responder_worker_bound
            .replace(transaction_responder_worker_bound);
        self
    }

    pub fn milestone_responder_worker_bound(mut self, milestone_responder_worker_bound: u32) -> Self {
        self.milestone_responder_worker_bound
            .replace(milestone_responder_worker_bound);
        self
    }

    pub fn transaction_requester_worker_bound(mut self, transaction_requester_worker_bound: u32) -> Self {
        self.transaction_requester_worker_bound
            .replace(transaction_requester_worker_bound);
        self
    }

    pub fn milestone_requester_worker_bound(mut self, milestone_requester_worker_bound: u32) -> Self {
        self.milestone_requester_worker_bound
            .replace(milestone_requester_worker_bound);
        self
    }

    pub fn receiver_worker_bound(mut self, receiver_worker_bound: u32) -> Self {
        self.receiver_worker_bound.replace(receiver_worker_bound);
        self
    }

    pub fn build(self) -> ProtocolConf {
        ProtocolConf {
            coo_public_key: self.coo_public_key.unwrap_or(CONF_COO_PUBLIC_KEY.1.to_owned()),
            coo_sponge_type: self.coo_sponge_type.unwrap_or(CONF_COO_SPONGE_TYPE.1.to_owned()),
            coo_security_level: self.coo_security_level.unwrap_or(CONF_COO_SECURITY.1),
            coo_depth: self.coo_depth.unwrap_or(CONF_COO_DEPTH.1),
            mwm: self.mwm.unwrap_or(CONF_MWM.1),
            milestone_request_send_worker_bound: self
                .milestone_request_send_worker_bound
                .unwrap_or(CONF_MILESTONE_REQUEST_SEND_WORKER_BOUND.1),
            transactionBroadcast_send_worker_bound: self
                .transaction_broadcast_send_worker_bound
                .unwrap_or(CONF_TRANSACTION_BROADCAST_SEND_WORKER_BOUND.1),
            transaction_request_send_worker_bound: self
                .transaction_request_send_worker_bound
                .unwrap_or(CONF_TRANSACTION_REQUEST_SEND_WORKER_BOUND.1),
            heartbeat_send_worker_bound: self
                .heartbeat_send_worker_bound
                .unwrap_or(CONF_HEARTBEAT_SEND_WORKER_BOUND.1),
            milestone_validator_worker_bound: self
                .milestone_validator_worker_bound
                .unwrap_or(CONF_MILESTONE_VALIDATOR_WORKER_BOUND.1),
            transaction_worker_bound: self.transaction_worker_bound.unwrap_or(CONF_TRANSACTION_WORKER_BOUND.1),
            transaction_responder_worker_bound: self
                .transaction_responder_worker_bound
                .unwrap_or(CONF_TRANSACTION_RESPONDER_WORKER_BOUND.1),
            milestone_responder_worker_bound: self
                .milestone_responder_worker_bound
                .unwrap_or(CONF_MILESTONE_RESPONDER_WORKER_BOUND.1),
            transaction_requester_worker_bound: self
                .transaction_requester_worker_bound
                .unwrap_or(CONF_TRANSACTION_REQUESTER_WORKER_BOUND.1),
            milestone_requester_worker_bound: self
                .milestone_requester_worker_bound
                .unwrap_or(CONF_MILESTONE_REQUESTER_WORKER_BOUND.1),
            receiver_worker_bound: self.receiver_worker_bound.unwrap_or(CONF_RECEIVER_WORKER_BOUND.1),
        }
    }
}

pub struct ProtocolConf {
    coo_public_key: String,
    // TODO sponge type type ?
    coo_sponge_type: String,
    coo_security_level: u8,
    coo_depth: u8,
    mwm: u8,
    milestone_request_send_worker_bound: u32,
    transactionBroadcast_send_worker_bound: u32,
    transaction_request_send_worker_bound: u32,
    heartbeat_send_worker_bound: u32,
    milestone_validator_worker_bound: u32,
    transaction_worker_bound: u32,
    transaction_responder_worker_bound: u32,
    milestone_responder_worker_bound: u32,
    transaction_requester_worker_bound: u32,
    milestone_requester_worker_bound: u32,
    receiver_worker_bound: u32,
}

// TODO move out of here
pub(crate) fn slice_eq(a: &[u8; 49], b: &[u8; 49]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    for i in 0..a.len() {
        if a[i] != b[i] {
            return false;
        }
    }

    true
}
