//! Route collection models and persistence for the API tester.
//!
//! This module defines HTTP route models, group normalization helpers, and
//! read/write support for the API tester TOML collection file.

use anyhow::Context;
use rand::{Rng, rng};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::{fmt, fs};

use crate::api_tester::paths;

/// Fallback route group used when a route group is blank.
pub const DEFAULT_ROUTE_GROUP: &str = "general";

/// Builds the default route group string.
///
/// # Returns
///
/// A [`String`] containing [`DEFAULT_ROUTE_GROUP`].
fn default_route_group() -> String {
    DEFAULT_ROUTE_GROUP.to_string()
}

/// Normalizes a route group name for storage.
///
/// Blank or whitespace-only values are replaced with
/// [`DEFAULT_ROUTE_GROUP`].
///
/// # Arguments
///
/// * `group` — Raw route group name.
///
/// # Returns
///
/// A normalized [`String`] group name.
fn normalize_group_name(group: &str) -> String {
    let trimmed = group.trim();

    if trimmed.is_empty() {
        default_route_group()
    } else {
        trimmed.to_string()
    }
}

/// Generates a random route scope identifier.
///
/// # Returns
///
/// A [`String`] in the format `route_<hex>`.
fn generate_scope_id() -> String {
    let mut random = rng();
    format!("route_{:08x}", random.random::<u32>())
}

/// HTTP methods supported by saved API tester routes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    /// HTTP GET method.
    Get,
    /// HTTP POST method.
    Post,
    /// HTTP PUT method.
    Put,
    /// HTTP PATCH method.
    Patch,
    /// HTTP DELETE method.
    Delete,
}

/// Formats an HTTP method as an uppercase label.
impl fmt::Display for HttpMethod {
    /// Formats an HTTP method as an uppercase string.
    ///
    /// # Arguments
    ///
    /// * `f` — Formatter used to write output.
    ///
    /// # Returns
    ///
    /// A [`fmt::Result`] indicating formatting success.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Patch => write!(f, "PATCH"),
            HttpMethod::Delete => write!(f, "DELETE"),
        }
    }
}

/// Parses a case-insensitive HTTP method label.
impl FromStr for HttpMethod {
    type Err = anyhow::Error;

    /// Parses an HTTP method string.
    ///
    /// # Arguments
    ///
    /// * `s` — Method string to parse.
    ///
    /// # Returns
    ///
    /// A parsed [`HttpMethod`] value.
    ///
    /// # Errors
    ///
    /// Returns an [`anyhow::Error`] if the method string is unsupported.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            "PUT" => Ok(HttpMethod::Put),
            "PATCH" => Ok(HttpMethod::Patch),
            "DELETE" => Ok(HttpMethod::Delete),
            _ => Err(anyhow::anyhow!("Invalid HTTP method: {}", s)),
        }
    }
}

/// Serializable route definition used by the API tester.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    /// Group name used to organize routes in the UI.
    #[serde(default = "default_route_group")]
    pub group: String,
    /// Stable route scope ID used for scoped variables.
    pub scope_id: String,
    /// Human-readable route name.
    pub name: String,
    /// HTTP method used by the route.
    pub method: HttpMethod,
    /// Request URL template.
    pub url: String,
    /// Request header templates.
    #[serde(default)]
    pub headers: Vec<String>,
    /// Optional request body template.
    #[serde(default)]
    pub body: Option<String>,
}

/// TOML root structure used for route collection persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CollectionFile {
    /// Persisted route groups.
    #[serde(default)]
    groups: Vec<RouteGroupFile>,
}

/// TOML group structure used for route collection persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RouteGroupFile {
    /// Group name.
    name: String,
    /// Routes contained by this group.
    #[serde(default)]
    routes: Vec<RouteFile>,
}

/// TOML route structure used for route collection persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RouteFile {
    /// Stable route scope ID.
    scope_id: String,
    /// Route name.
    name: String,
    /// HTTP method.
    method: HttpMethod,
    /// URL template.
    url: String,
    /// Header templates.
    #[serde(default)]
    headers: Vec<String>,
    /// Optional body template.
    #[serde(default)]
    body: Option<String>,
}

/// In-memory collection of API tester routes.
#[derive(Debug, Clone)]
pub struct Collection {
    /// Flattened route list rendered and edited by the UI.
    pub routes: Vec<Route>,
}

impl Collection {
    /// Loads route definitions from the persisted collection file.
    ///
    /// # Returns
    ///
    /// A [`Collection`] parsed from disk, or an empty collection when the file
    /// does not exist.
    ///
    /// # Errors
    ///
    /// Returns an [`anyhow::Error`] if the collection file cannot be read or
    /// parsed.
    pub fn load() -> anyhow::Result<Self> {
        let path = paths::collection_path();

        if !path.exists() {
            return Ok(Self { routes: vec![] });
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read collection file: {}", path.display()))?;
        let collection: CollectionFile = toml::from_str(&content)
            .with_context(|| format!("failed to parse collection TOML: {}", path.display()))?;

        let mut routes = vec![];

        for group in collection.groups {
            let group_name = normalize_group_name(&group.name);

            for route in group.routes {
                routes.push(Route {
                    group: group_name.clone(),
                    scope_id: route.scope_id,
                    name: route.name,
                    method: route.method,
                    url: route.url,
                    headers: route.headers,
                    body: route.body,
                });
            }
        }

        Ok(Self { routes })
    }

    /// Saves route definitions to the persisted collection file.
    ///
    /// # Returns
    ///
    /// An empty [`anyhow::Result`] on success.
    ///
    /// # Errors
    ///
    /// Returns an [`anyhow::Error`] if directory creation, serialization, or
    /// file writing fails.
    pub fn save(&self) -> anyhow::Result<()> {
        let path = paths::collection_path();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "failed to create collection directory: {}",
                    parent.display()
                )
            })?;
        }

        let mut groups: Vec<RouteGroupFile> = vec![];

        for route in &self.routes {
            let group_name = normalize_group_name(&route.group);
            let route_file = RouteFile {
                scope_id: route.scope_id.clone(),
                name: route.name.clone(),
                method: route.method.clone(),
                url: route.url.clone(),
                headers: route.headers.clone(),
                body: route.body.clone(),
            };

            if let Some(group) = groups.iter_mut().find(|group| group.name == group_name) {
                group.routes.push(route_file);
            } else {
                groups.push(RouteGroupFile {
                    name: group_name,
                    routes: vec![route_file],
                });
            }
        }

        let serialized = toml::to_string_pretty(&CollectionFile { groups })
            .context("failed to serialize collection")?;

        fs::write(&path, serialized)
            .with_context(|| format!("failed to write collection file: {}", path.display()))?;

        Ok(())
    }

    /// Lists unique route group names in sorted order.
    ///
    /// # Returns
    ///
    /// A sorted [`Vec`] of distinct group names.
    pub fn group_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .routes
            .iter()
            .map(|r| r.group.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        names.sort();
        names
    }

    /// Appends a new route after applying collection invariants.
    ///
    /// # Arguments
    ///
    /// * `route` — Route definition to append.
    pub fn add_route(&mut self, mut route: Route) {
        route.group = normalize_group_name(&route.group);
        if route.scope_id.trim().is_empty() {
            route.scope_id = self.new_scope_id();
        }
        self.routes.push(route);
    }

    /// Replaces an existing route at the provided index.
    ///
    /// # Arguments
    ///
    /// * `index` — Route index to replace.
    /// * `route` — Updated route definition.
    ///
    /// # Returns
    ///
    /// An empty [`anyhow::Result`] on success.
    ///
    /// # Errors
    ///
    /// Returns an [`anyhow::Error`] if `index` is out of bounds.
    pub fn update_route(&mut self, index: usize, mut route: Route) -> anyhow::Result<()> {
        if index >= self.routes.len() {
            anyhow::bail!("route index out of bounds: {index}");
        }

        route.group = normalize_group_name(&route.group);
        if route.scope_id.trim().is_empty() {
            route.scope_id = self.new_scope_id();
        }
        self.routes[index] = route;
        Ok(())
    }

    /// Removes a route at the provided index.
    ///
    /// # Arguments
    ///
    /// * `index` — Route index to delete.
    ///
    /// # Returns
    ///
    /// An empty [`anyhow::Result`] on success.
    ///
    /// # Errors
    ///
    /// Returns an [`anyhow::Error`] if `index` is out of bounds.
    pub fn delete_route(&mut self, index: usize) -> anyhow::Result<()> {
        if index >= self.routes.len() {
            anyhow::bail!("route index out of bounds: {index}");
        }

        self.routes.remove(index);
        Ok(())
    }

    /// Generates a unique scope ID not used by existing routes.
    ///
    /// # Returns
    ///
    /// A unique route scope identifier [`String`].
    pub fn new_scope_id(&self) -> String {
        loop {
            let candidate = generate_scope_id();

            if !self.routes.iter().any(|route| route.scope_id == candidate) {
                return candidate;
            }
        }
    }
}
