use std::path::PathBuf;

const COLLECTION_FILE: &str = "collections.toml";
const DATA_DIR: &str = "dev-tools/api-tester";
const VARIABLES_FILE: &str = "api-tester.env";
const COOKIE_JAR_FILE: &str = "cookies.txt";

fn workspace_root() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path
}

fn data_dir() -> PathBuf {
    workspace_root().join(DATA_DIR)
}

pub fn collection_path() -> PathBuf {
    workspace_root().join(COLLECTION_FILE)
}

pub fn variables_path() -> PathBuf {
    workspace_root().join(VARIABLES_FILE)
}

pub fn cookie_jar_path() -> PathBuf {
    data_dir().join(COOKIE_JAR_FILE)
}
