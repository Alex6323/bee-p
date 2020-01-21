pub struct BeeTransaction {
    pub signature_or_message: Vec<u8>,
    //pub address: diesel::sql_types::Binary,
    pub value: i32,
    /*pub obsolete_tag: diesel::sql_types::Binary,
    pub timestamp: i32,
    pub current_index: i16,
    pub last_index: i16,
    pub bundle: diesel::sql_types::Binary,
    pub trunk: diesel::sql_types::Binary,
    pub branch: diesel::sql_types::Binary,
    pub tag: diesel::sql_types::Binary,
    pub attachment_timestamp: i32,
    pub attachment_timestamp_lower: i32,
    pub attachment_timestamp_upper: i32,
    pub nonce: diesel::sql_types::Binary,
    pub hash: diesel::sql_types::Binary,
    pub snapshot_index: i32,
    pub solid: bool,
    pub validity: bool,
    pub arrival_timestamp: i32,*/
}

pub struct BeeMilestone {
    pub id: i32,
    //pub hash: diesel::sql_types::Binary,
    //pub delta: diesel::sql_types::Binary,
}
