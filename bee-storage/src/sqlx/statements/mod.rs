use itertools::Itertools;

pub const TRANSACTION_COL_HASH: &str = "hash";
pub const TRANSACTION_COL_VALUE: &str = "value";
pub const TRANSACTION_COL_CURRENT_INDEX: &str = "current_index";
pub const TRANSACTION_COL_LAST_INDEX: &str = "last_index";
pub const TRANSACTION_COL_TAG: &str = "tag";
pub const TRANSACTION_COL_BUNDLE: &str = "bundle";
pub const TRANSACTION_COL_ADDRESS: &str = "address";
pub const TRANSACTION_COL_TRUNK: &str = "trunk";
pub const TRANSACTION_COL_BRANCH: &str = "branch";
pub const TRANSACTION_COL_NONCE: &str = "nonce";
pub const TRANSACTION_COL_OBSOLETE_TAG: &str = "obsolete_tag";
pub const TRANSACTION_COL_PAYLOAD: &str = "payload";
pub const TRANSACTION_COL_TIMESTAMP: &str = "timestamp";
pub const TRANSACTION_COL_ATTACHMENT_TIMESTAMP: &str = "attachment_timestamp";
pub const TRANSACTION_COL_ATTACHMENT_TIMESTAMP_UPPER: &str = "attachment_timestamp_upper";
pub const TRANSACTION_COL_ATTACHMENT_TIMESTAMP_LOWER: &str = "attachment_timestamp_lower";
pub const TRANSACTION_COL_SOLID: &str = "solid";
pub const TRANSACTION_COL_SNAPSHOT_INDEX: &str = "snapshot_index";

pub const MILESTONE_COL_ID: &str = "id";
pub const MILESTONE_COL_HASH: &str = "hash";
pub const MILESTONE_COL_DELTA: &str = "delta";

pub const INSERT_TRANSACTION_STATEMENT: &str = r#"
        INSERT INTO transactions (payload, address, value, obsolete_tag, timestamp, current_index, last_index, bundle, trunk, branch, tag
        ,attachment_timestamp, attachment_timestamp_lower, attachment_timestamp_upper, nonce, hash)
        VALUES ( $1, $2 , $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
                "#;

pub const FIND_TRANSACTION_BY_HASH_STATEMENT: &str = r#"
SELECT payload, address, value, obsolete_tag, timestamp, current_index, last_index, bundle, trunk, branch, tag
        ,attachment_timestamp, attachment_timestamp_lower, attachment_timestamp_upper, nonce, hash
FROM transactions
WHERE hash=$1
        "#;

pub const SELECT_HASH_BRANCH_TRUNK_LIMIT_STATEMENT: &str = r#"
SELECT hash, branch, trunk
FROM transactions
 OFFSET $1 LIMIT $2
        "#;

// NOTE: Currently this constant is not used, and throws a warning.
// const SELECT_HASH_LIMIT_STATEMENT: &str =         r#"
// SELECT hash
// FROM transactions
// OFFSET $1 LIMIT $2
// RETURNING hash
// "#;

pub const UPDATE_SNAPSHOT_INDEX_STATEMENT: &str = r#"UPDATE transactions set snapshot_index =$1 WHERE hash=$2"#;

pub const UPDATE_SET_SOLID_STATEMENT: &str = r#"UPDATE transactions set solid=1 WHERE hash=$1"#;

pub const DELETE_TRANSACTION_STATEMENT: &str = r#"DELETE FROM transactions WHERE hash =$1"#;

pub const INSERT_MILESTONE_STATEMENT: &str = r#"
        INSERT INTO milestones (id, hash)
        VALUES ($1, $2)
                "#;

pub const FIND_MILESTONE_BY_HASH_STATEMENT: &str = r#"
SELECT id, hash
FROM milestones
WHERE hash=$1
        "#;

pub const DELETE_MILESTONE_BY_HASH_STATEMENT: &str = r#"DELETE FROM milestones WHERE hash =$1"#;

pub const STORE_DELTA_STATEMENT: &str = r#"UPDATE milestones SET delta =$1 WHERE id =$2"#;

pub const LOAD_DELTA_STATEMENT_BY_INDEX: &str = r#"
SELECT delta
FROM milestones
WHERE id=$1
        "#;

pub fn select_solid_states_by_hashes_statement(num_hashes: usize) -> String {
    format!(
        r#"
                SELECT solid
                FROM transactions
                WHERE hash in ({})"#,
        placeholders(num_hashes, 1)
    )
}

pub fn select_snapshot_indexes_by_hashes_statement(num_hashes: usize) -> String {
    format!(
        r#"
                SELECT snapshot_index
                FROM transactions
                WHERE hash in ({})"#,
        placeholders(num_hashes, 1)
    )
}

fn placeholders(rows: usize, columns: usize) -> String {
    (0..rows)
        .format_with(",", |i, f| {
            f(&format_args!(
                "({})",
                (1..=columns).format_with(",", |j, f| f(&format_args!("${}", j + (i * columns))))
            ))
        })
        .to_string()
}
