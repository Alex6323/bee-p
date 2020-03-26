CREATE TABLE IF NOT EXISTS transactions (
  payload BYTEA NOT NULL,
  address BYTEA NOT NULL,
  value INTEGER NOT NULL,
  obsolete_tag BYTEA,
  timestamp INTEGER NOT NULL,
  current_index SMALLINT NOT NULL,
  last_index SMALLINT NOT NULL,
  bundle BYTEA NOT NULL,
  trunk BYTEA NOT NULL,
  branch BYTEA NOT NULL,
  tag BYTEA NOT NULL,
  attachment_timestamp INTEGER NOT NULL,
  attachment_timestamp_lower INTEGER NOT NULL,
  attachment_timestamp_upper INTEGER NOT NULL,
  nonce BYTEA NOT NULL,
  hash BYTEA NOT NULL PRIMARY KEY,
  snapshot_index INTEGER NOT NULL DEFAULT 0,
  solid SMALLINT NOT NULL DEFAULT 0,
  validity SMALLINT NOT NULL DEFAULT 0
);


CREATE TABLE IF NOT EXISTS  milestones (
  id INTEGER NOT NULL PRIMARY KEY,
  hash BYTEA NOT NULL UNIQUE,
  delta BYTEA
);