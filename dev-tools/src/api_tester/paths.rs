use std::path::PathBuf;

const DATA_DIR: &str = ".api-tester";
const COLLECTION_FILE: &str = "collections.toml";
const VARIABLES_FILE: &str = "variables.toml";
const COOKIE_JAR_FILE: &str = "cookies.txt";
const ROUTE_LIST_STATE_FILE: &str = "route-list-state.toml";

fn workspace_root() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path
}

fn data_dir() -> PathBuf {
    workspace_root().join(DATA_DIR)
}

pub fn collection_path() -> PathBuf {
    data_dir().join(COLLECTION_FILE)
}

pub fn variables_path() -> PathBuf {
    data_dir().join(VARIABLES_FILE)
}

pub fn cookie_jar_path() -> PathBuf {
    data_dir().join(COOKIE_JAR_FILE)
}

pub fn route_list_state_path() -> PathBuf {
    data_dir().join(ROUTE_LIST_STATE_FILE)
}
