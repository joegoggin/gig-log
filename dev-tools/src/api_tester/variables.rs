use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

use crate::api_tester::paths;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variables {
    variables: HashMap<String, String>,
}

impl Variables {
    pub fn load() -> anyhow::Result<Self> {
        let path = paths::variables_path();

        if !path.exists() {
            return Ok(Self {
                variables: HashMap::<String, String>::new(),
            });
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read variables file: {}", path.display()))?;

        let mut variables = HashMap::new();

        for (index, raw_line) in content.lines().enumerate() {
            let line_number = index + 1;
            let line = raw_line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let (raw_key, raw_value) = line
                .split_once('=')
                .ok_or_else(|| anyhow::anyhow!("invalid env line {line_number}: {line}"))?;

            let key = raw_key.trim();
            if key.is_empty() {
                anyhow::bail!("empty key on env line {line_number}");
            }

            let mut value = raw_value.trim().to_string();
            if value.len() >= 2
                && ((value.starts_with('"') && value.ends_with('"'))
                    || (value.starts_with('\'') && value.ends_with('\'')))
            {
                value = value[1..value.len() - 1].to_string();
            }

            variables.insert(key.to_string(), value);
        }

        Ok(Self { variables })
    }

    pub fn substitute(&self, template: &str) -> String {
        let mut output = String::with_capacity(template.len());
        let mut rest = template;

        while let Some(start) = rest.find("{{") {
            output.push_str(&rest[..start]);
            let after_open = &rest[start + 2..];

            match after_open.find("}}") {
                Some(end) => {
                    let token = &after_open[..end];
                    let key = token.trim();

                    if let Some(value) = self.variables.get(key) {
                        output.push_str(value);
                    } else {
                        output.push_str("{{");
                        output.push_str(token);
                        output.push_str("}}");
                    }

                    rest = &after_open[end + 2..];
                }
                None => {
                    output.push_str(&rest[start..]);
                    rest = "";
                }
            }
        }

        output.push_str(rest);
        output
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = paths::variables_path();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create variables directory: {}", parent.display())
            })?;
        }

        let mut entries: Vec<_> = self.variables.iter().collect();
        entries.sort_by(|(left, _), (right, _)| left.cmp(right));

        let mut content = String::new();
        for (key, value) in entries {
            let escaped = value.replace('"', "\\\"");
            let needs_quotes = escaped.chars().any(char::is_whitespace) || escaped.contains('#');

            if needs_quotes {
                content.push_str(&format!("{key}=\"{escaped}\"\n"));
            } else {
                content.push_str(&format!("{key}={escaped}\n"));
            }
        }

        fs::write(&path, content)
            .with_context(|| format!("failed to write variables file: {}", path.display()))?;

        Ok(())
    }

    pub fn add(&mut self, key: String, value: String) {
        self.variables.insert(key, value);
    }

    pub fn delete(&mut self, key: &str) {
        self.variables.remove(key);
    }

    pub fn entries(&self) -> Vec<(&String, &String)> {
        let mut entries: Vec<_> = self.variables.iter().collect();
        entries.sort_by(|(a, _), (b, _)| a.cmp(b));
        entries
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.variables.get(key)
    }
}
