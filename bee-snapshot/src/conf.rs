use serde::Deserialize;

const CONF_META_FILE_PATH: &str = "./data/mainnet.snapshot.meta";
const CONF_STATE_FILE_PATH: &str = "./data/mainnet.snapshot.state";

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotConfBuilder {
    meta_file_path: Option<String>,
    state_file_path: Option<String>,
}

impl SnapshotConfBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn meta_file_path(mut self, meta_file_path: String) -> Self {
        self.meta_file_path.replace(meta_file_path);
        self
    }

    pub fn state_file_path(mut self, state_file_path: String) -> Self {
        self.state_file_path.replace(state_file_path);
        self
    }

    pub fn build(self) -> SnapshotConf {
        SnapshotConf {
            meta_file_path: self.meta_file_path.unwrap_or(CONF_META_FILE_PATH.to_string()),
            state_file_path: self.state_file_path.unwrap_or(CONF_STATE_FILE_PATH.to_string()),
        }
    }
}

#[derive(Clone)]
pub struct SnapshotConf {
    meta_file_path: String,
    state_file_path: String,
}

impl SnapshotConf {
    pub fn meta_file_path(&self) -> &String {
        &self.meta_file_path
    }

    pub fn state_file_path(&self) -> &String {
        &self.state_file_path
    }
}
