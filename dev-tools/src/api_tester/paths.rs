//! Filesystem path helpers for API tester persistence.
//!
//! This module centralizes file locations for route collections, variables,
//! cookies, and route list UI state under the workspace-level `.api-tester`
//! directory.

use std::path::PathBuf;

/// Name of the API tester data directory at workspace root.
const DATA_DIR: &str = ".api-tester";
/// Route collection TOML filename.
const COLLECTION_FILE: &str = "collections.toml";
/// Variable storage TOML filename.
const VARIABLES_FILE: &str = "variables.toml";
/// Cookie jar filename used by `curl` executions.
const COOKIE_JAR_FILE: &str = "cookies.txt";
/// Route list UI state TOML filename.
const ROUTE_LIST_STATE_FILE: &str = "route-list-state.toml";

/// Resolves the workspace root path from the crate manifest directory.
///
/// # Returns
///
/// A [`PathBuf`] pointing to the workspace root.
fn workspace_root() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path
}

/// Resolves the API tester data directory path.
///
/// # Returns
///
/// A [`PathBuf`] to the `.api-tester` directory.
fn data_dir() -> PathBuf {
    workspace_root().join(DATA_DIR)
}

/// Resolves the route collection file path.
///
/// # Returns
///
/// A [`PathBuf`] to `collections.toml`.
pub fn collection_path() -> PathBuf {
    data_dir().join(COLLECTION_FILE)
}

/// Resolves the variables file path.
///
/// # Returns
///
/// A [`PathBuf`] to `variables.toml`.
pub fn variables_path() -> PathBuf {
    data_dir().join(VARIABLES_FILE)
}

/// Resolves the cookie jar path used by request execution.
///
/// # Returns
///
/// A [`PathBuf`] to `cookies.txt`.
pub fn cookie_jar_path() -> PathBuf {
    data_dir().join(COOKIE_JAR_FILE)
}

/// Resolves the persisted route list state file path.
///
/// # Returns
///
/// A [`PathBuf`] to `route-list-state.toml`.
pub fn route_list_state_path() -> PathBuf {
    data_dir().join(ROUTE_LIST_STATE_FILE)
}
