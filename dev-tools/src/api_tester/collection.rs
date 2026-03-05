use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::str::FromStr;
use std::{fmt, fs};

pub const COLLECTION_PATH: &str = "dev-tools/api-tester/collections.toml";

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
    pub name: String,
    pub method: HttpMethod,
    pub url: String,
    #[serde(default)]
    pub headers: Vec<String>,
    #[serde(default)]
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub routes: Vec<Route>,
}

impl Collection {
    pub fn load() -> anyhow::Result<Self> {
        let path = Path::new(COLLECTION_PATH);

        if !path.exists() {
            return Ok(Self { routes: vec![] });
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("failed to read collection file: {}", path.display()))?;
        let collection = toml::from_str(&content)
            .with_context(|| format!("failed to parse collection TOML: {}", path.display()))?;

        Ok(collection)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Path::new(COLLECTION_PATH);
        let serialized = toml::to_string_pretty(self).context("failed to serialize collection")?;

        fs::write(path, serialized)
            .with_context(|| format!("failed to write collection file: {}", path.display()))?;

        Ok(())
    }

    pub fn add_route(&mut self, route: Route) {
        self.routes.push(route);
    }

    pub fn update_route(&mut self, index: usize, route: Route) -> anyhow::Result<()> {
        if index >= self.routes.len() {
            anyhow::bail!("route index out of bounds: {index}");
        }

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
