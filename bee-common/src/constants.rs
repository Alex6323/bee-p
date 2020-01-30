pub const ENV_VAR: &str = "BEE";
pub const DEBUG: &str = "debug";
pub const CONFIG: &str = "./config";

pub const TRYTE_ZERO: char = '9';

pub const TRANSACTION_TRIT_LEN: usize = 8019;
pub const TRANSACTION_TRYT_LEN: usize = TRANSACTION_TRIT_LEN / 3; //2673
pub const TRANSACTION_BYTE_LEN: usize = TRANSACTION_TRIT_LEN / 5 + 1; //1604

pub const PAYLOAD_TRIT_LEN: usize = 6561;
pub const ADDRESS_TRIT_LEN: usize = 243;
pub const VALUE_TRIT_LEN: usize = 81;
pub const TAG_TRIT_LEN: usize = 81;
pub const TIMESTAMP_TRIT_LEN: usize = 27;
pub const INDEX_TRIT_LEN: usize = 27;
pub const HASH_TRIT_LEN: usize = 243;
pub const NONCE_TRIT_LEN: usize = 81;

pub const MAINNET_DIFFICULTY: usize = 14;
pub const DEVNET_DIFFICULTY: usize = 9;
pub const SPAMNET_DIFFICULTY: usize = 6;
