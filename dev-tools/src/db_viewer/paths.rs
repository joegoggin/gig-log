use std::path::PathBuf;

const DATA_DIR: &str = ".db-viewer";
const STATE_FILE: &str = "state.toml";
const QUERY_FILE: &str = "query.sql";

fn workspace_root() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path
}

fn data_dir() -> PathBuf {
    workspace_root().join(DATA_DIR)
}

pub fn state_path() -> PathBuf {
    data_dir().join(STATE_FILE)
}

pub fn query_path() -> PathBuf {
    data_dir().join(QUERY_FILE)
}
