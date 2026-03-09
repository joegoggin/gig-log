use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::{fmt, fs};

use crate::api_tester::paths;

pub const DEFAULT_ROUTE_GROUP: &str = "general";

fn default_route_group() -> String {
    DEFAULT_ROUTE_GROUP.to_string()
}

fn normalize_group_name(group: &str) -> String {
    let trimmed = group.trim();

    if trimmed.is_empty() {
        default_route_group()
    } else {
        trimmed.to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl fmt::Display for HttpMethod {
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

impl FromStr for HttpMethod {
    type Err = anyhow::Error;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    #[serde(default = "default_route_group")]
    pub group: String,
    pub name: String,
    pub method: HttpMethod,
    pub url: String,
    #[serde(default)]
    pub headers: Vec<String>,
    #[serde(default)]
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CollectionFile {
    #[serde(default)]
    groups: Vec<RouteGroupFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RouteGroupFile {
    name: String,
    #[serde(default)]
    routes: Vec<RouteFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RouteFile {
    name: String,
    method: HttpMethod,
    url: String,
    #[serde(default)]
    headers: Vec<String>,
    #[serde(default)]
    body: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Collection {
    pub routes: Vec<Route>,
}

impl Collection {
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

    pub fn add_route(&mut self, mut route: Route) {
        route.group = normalize_group_name(&route.group);
        self.routes.push(route);
    }

    pub fn update_route(&mut self, index: usize, mut route: Route) -> anyhow::Result<()> {
        if index >= self.routes.len() {
            anyhow::bail!("route index out of bounds: {index}");
        }

        route.group = normalize_group_name(&route.group);
        self.routes[index] = route;
        Ok(())
    }

    pub fn delete_route(&mut self, index: usize) -> anyhow::Result<()> {
        if index >= self.routes.len() {
            anyhow::bail!("route index out of bounds: {index}");
        }

        self.routes.remove(index);
        Ok(())
    }
}
