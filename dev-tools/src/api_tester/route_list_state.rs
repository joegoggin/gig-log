//! Persistence model for route list expansion and selection state.
//!
//! This module stores which route groups are expanded and which route or group
//! is currently selected so the route list can restore UI context across runs.

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::api_tester::{collection::Route, paths};

/// Serializable route identity used to restore route selection.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct RouteSelection {
    /// Normalized group name.
    pub group: String,
    /// Route display name.
    pub name: String,
    /// HTTP method label.
    pub method: String,
    /// Route URL template.
    pub url: String,
}

impl RouteSelection {
    /// Builds a persisted selection payload from a concrete route.
    ///
    /// # Arguments
    ///
    /// * `route` — Route to snapshot.
    ///
    /// # Returns
    ///
    /// A [`RouteSelection`] that can be persisted and compared later.
    pub fn from_route(route: &Route) -> Self {
        let group = if route.group.trim().is_empty() {
            crate::api_tester::collection::DEFAULT_ROUTE_GROUP.to_string()
        } else {
            route.group.clone()
        };

        Self {
            group,
            name: route.name.clone(),
            method: route.method.to_string(),
            url: route.url.clone(),
        }
    }
}

/// Selected row kind persisted by the route list.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SelectedItem {
    /// Selected route row.
    Route(RouteSelection),
    /// Selected group header row.
    Group {
        /// Stores the selected group header name.
        name: String,
    },
}

/// Persisted state for route list expansion and selection.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct RouteListState {
    /// Expanded route group names.
    #[serde(default)]
    pub expanded_groups: Vec<String>,
    /// Currently selected route or group.
    #[serde(default)]
    pub selected: Option<SelectedItem>,
}

impl RouteListState {
    /// Loads route list state from the default API tester path.
    ///
    /// # Returns
    ///
    /// A [`RouteListState`] loaded from disk, or default state on failure.
    pub fn load() -> Self {
        Self::load_or_default(&paths::route_list_state_path())
    }

    /// Loads route list state from a path, defaulting on read/parse failure.
    ///
    /// # Arguments
    ///
    /// * `path` — Route list state file path.
    ///
    /// # Returns
    ///
    /// A [`RouteListState`] parsed from disk or [`RouteListState::default`]
    /// when loading fails.
    pub fn load_or_default(path: &Path) -> Self {
        match Self::load_from(path) {
            Ok(state) => state,
            Err(error) => {
                eprintln!(
                    "Warning: failed to load route list state from {}: {error}",
                    path.display()
                );
                Self::default()
            }
        }
    }

    /// Loads route list state from a specific path.
    ///
    /// # Arguments
    ///
    /// * `path` — Route list state file path.
    ///
    /// # Returns
    ///
    /// A parsed [`RouteListState`] value.
    ///
    /// # Errors
    ///
    /// Returns an [`anyhow::Error`] if the file cannot be read or parsed.
    pub fn load_from(path: &Path) -> anyhow::Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("failed to read route list state file: {}", path.display()))?;

        toml::from_str(&content)
            .with_context(|| format!("failed to parse route list state TOML: {}", path.display()))
    }

    /// Saves route list state to the default API tester path.
    ///
    /// # Returns
    ///
    /// An empty [`anyhow::Result`] on success.
    ///
    /// # Errors
    ///
    /// Returns an [`anyhow::Error`] if serialization or writing fails.
    pub fn save(&self) -> anyhow::Result<()> {
        self.save_to(&paths::route_list_state_path())
    }

    /// Saves route list state to a specific path.
    ///
    /// # Arguments
    ///
    /// * `path` — Destination file path.
    ///
    /// # Returns
    ///
    /// An empty [`anyhow::Result`] on success.
    ///
    /// # Errors
    ///
    /// Returns an [`anyhow::Error`] if directory creation, serialization, or
    /// writing fails.
    pub fn save_to(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "failed to create route list state directory: {}",
                    parent.display()
                )
            })?;
        }

        let serialized = toml::to_string_pretty(&self.normalized())
            .context("failed to serialize route list state")?;

        fs::write(path, serialized)
            .with_context(|| format!("failed to write route list state file: {}", path.display()))
    }

    /// Produces a normalized copy suitable for stable persistence.
    ///
    /// # Returns
    ///
    /// A normalized [`RouteListState`] with trimmed, deduplicated values.
    fn normalized(&self) -> Self {
        let mut state = self.clone();

        state.expanded_groups = state
            .expanded_groups
            .iter()
            .map(|group| group.trim().to_string())
            .filter(|group| !group.is_empty())
            .collect();
        state.expanded_groups.sort();
        state.expanded_groups.dedup();

        state.selected = state.selected.map(|selected| match selected {
            SelectedItem::Route(mut route) => {
                route.group = route.group.trim().to_string();
                route.name = route.name.trim().to_string();
                route.method = route.method.trim().to_uppercase();
                route.url = route.url.trim().to_string();
                SelectedItem::Route(route)
            }
            SelectedItem::Group { name } => SelectedItem::Group {
                name: name.trim().to_string(),
            },
        });

        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_and_load_round_trip() {
        let temp_dir = tempfile::tempdir().expect("creates temp dir");
        let path = temp_dir.path().join("route-list-state.toml");

        let state = RouteListState {
            expanded_groups: vec!["group-b".to_string(), "group-a".to_string()],
            selected: Some(SelectedItem::Route(RouteSelection {
                group: "group-a".to_string(),
                name: "list users".to_string(),
                method: "get".to_string(),
                url: "https://example.com/users".to_string(),
            })),
        };

        state.save_to(&path).expect("saves state");
        let loaded = RouteListState::load_from(&path).expect("loads state");

        assert_eq!(
            loaded,
            RouteListState {
                expanded_groups: vec!["group-a".to_string(), "group-b".to_string()],
                selected: Some(SelectedItem::Route(RouteSelection {
                    group: "group-a".to_string(),
                    name: "list users".to_string(),
                    method: "GET".to_string(),
                    url: "https://example.com/users".to_string(),
                })),
            }
        );
    }

    #[test]
    fn invalid_file_returns_default_in_load_or_default() {
        let temp_dir = tempfile::tempdir().expect("creates temp dir");
        let path = temp_dir.path().join("route-list-state.toml");
        fs::write(&path, "not = [valid").expect("writes invalid file");

        let loaded = RouteListState::load_or_default(&path);

        assert_eq!(loaded, RouteListState::default());
    }
}
