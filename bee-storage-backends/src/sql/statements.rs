const TRANSACTION_COL_HASH: &str = "hash";
const TRANSACTION_COL_VALUE: &str = "value";
const TRANSACTION_COL_CURRENT_INDEX: &str = "current_index";
const TRANSACTION_COL_LAST_INDEX: &str = "last_index";
const TRANSACTION_COL_TAG: &str = "tag";
const TRANSACTION_COL_BUNDLE: &str = "bundle";
const TRANSACTION_COL_ADDRESS: &str = "address";
const TRANSACTION_COL_TRUNK: &str = "trunk";
const TRANSACTION_COL_BRANCH: &str = "branch";
const TRANSACTION_COL_NONCE: &str = "nonce";
const TRANSACTION_COL_OBSOLETE_TAG: &str = "obsolete_tag";
const TRANSACTION_COL_SIG_OR_MESSAGE: &str = "signature_or_message";
const TRANSACTION_COL_TIMESTAMP: &str = "timestamp";
const TRANSACTION_COL_ATTACHMENT_TIMESTAMP: &str = "attachment_timestamp";
const TRANSACTION_COL_ATTACHMENT_TIMESTAMP_UPPER: &str = "attachment_timestamp_upper";
const TRANSACTION_COL_ATTACHMENT_TIMESTAMP_LOWER: &str = "attachment_timestamp_lower";

const MILESTONE_COL_ID: &str = "id";
const MILESTONE_COL_HASH: &str = "hash";
const MILESTONE_COL_DELTA: &str = "delta";

const INSERT_TRANSACTION_STATEMENT : &str =            r#"
        INSERT INTO transactions (signature_or_message, address, value, obsolete_tag, timestamp, current_index, last_index, bundle, trunk, branch, tag
        ,attachment_timestamp, attachment_timestamp_lower, attachment_timestamp_upper, nonce, hash)
        VALUES ( $1, $2 , $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
        RETURNING hash
                "#;

const FIND_TRANSACTION_BY_HASH_STATEMENT : &str =         r#"
SELECT signature_or_message, address, value, obsolete_tag, timestamp, current_index, last_index, bundle, trunk, branch, tag
        ,attachment_timestamp, attachment_timestamp_lower, attachment_timestamp_upper, nonce, hash
FROM transactions
WHERE hash=$1
        "#;


const SELECT_HASH_BRANCH_TRUNK_LIMIT_STATEMENT: &str =         r#"
SELECT hash, branch, trunk
FROM transactions
LIMIT $1, $2
RETURNING hash, branch, trunk
        "#;

const SELECT_HAS_LIMIT_STATEMENT: &str =         r#"
SELECT hash
FROM transactions
LIMIT $1, $2
RETURNING hash
        "#;

const UPDATE_SNAPSHOT_INDEX_STATEMENT: &str =   r#"UPDATE transactions set snapshot_index =$1 WHERE hash hash=$2"#;

const UPDATE_SET_SOLID_STATEMENT: &str =   r#"UPDATE transactions set snapshot_index =$1 WHERE hash=$2"#;

const DELETE_TRANSACTION_STATEMENT: &str = r#"DELETE FROM  transactions WHERE hash =$1"#;

const INSERT_MILESTONE_STATEMENT: &str = r#"
        INSERT INTO milestones (id, hash)
        VALUES ($1, $2)
        RETURNING id
                "#;

const FIND_MILESTONE_BY_HASH_STATEMENT : &str =         r#"
SELECT id, hash
FROM milestones
WHERE hash=$1
        "#;

const DELETE_MILESTONE_BY_HASH_STATEMENT : &str = r#""DELETE FROM  milestones WHERE hash =$1"#;

const STORE_DELTA_STATEMENT: &str = r#"UPDATE milestones SET delta =$1 WHERE id =$2"#;

const LOAD_DELTA_STATEMENT_BY_INDEX: &str = r#"
SELECT hash,delta
FROM milestones
WHERE id = $1
RETURNING
hash, delta"#;
