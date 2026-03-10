use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::fs;

use crate::api_tester::paths;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum VariableMode {
    Hidden,
    #[default]
    Placeholder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Variable {
    value: String,
    #[serde(default)]
    mode: VariableMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct VariablesFile {
    #[serde(default)]
    variables: BTreeMap<String, Variable>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Variables {
    variables: BTreeMap<String, Variable>,
}

impl Variables {
    pub fn load() -> anyhow::Result<Self> {
        let path = paths::variables_path();

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read variables file: {}", path.display()))?;

        Self::parse(&content)
            .with_context(|| format!("failed to parse variables file: {}", path.display()))
    }

    fn parse(content: &str) -> anyhow::Result<Self> {
        let variables_file: VariablesFile =
            toml::from_str(content).context("failed to parse variables TOML")?;

        Ok(Self {
            variables: variables_file.variables,
        })
    }

    pub fn substitute(&self, template: &str) -> String {
        self.substitute_for_execution(template)
    }

    pub fn substitute_for_execution(&self, template: &str) -> String {
        self.substitute_with(template, |variable| variable.value.as_str())
    }

    pub fn substitute_for_preview(&self, template: &str) -> String {
        self.substitute_with(template, |variable| match variable.mode {
            VariableMode::Hidden => "hidden",
            VariableMode::Placeholder => variable.value.as_str(),
        })
    }

    pub fn redact_hidden_values(&self, text: &str) -> String {
        let mut hidden_values: Vec<&str> = self
            .variables
            .values()
            .filter(|variable| variable.mode == VariableMode::Hidden)
            .map(|variable| variable.value.as_str())
            .filter(|value| !value.is_empty())
            .collect();

        hidden_values.sort_by_key(|value| Reverse(value.len()));

        hidden_values
            .into_iter()
            .fold(text.to_string(), |output, hidden_value| {
                output.replace(hidden_value, "hidden")
            })
    }

    fn substitute_with<F>(&self, template: &str, mut replacement_for: F) -> String
    where
        F: for<'a> FnMut(&'a Variable) -> &'a str,
    {
        let mut output = String::with_capacity(template.len());
        let mut rest = template;

        while let Some(start) = rest.find("{{") {
            output.push_str(&rest[..start]);
            let after_open = &rest[start + 2..];

            match after_open.find("}}") {
                Some(end) => {
                    let token = &after_open[..end];
                    let key = token.trim();

                    if let Some(variable) = self.variables.get(key) {
                        output.push_str(replacement_for(variable));
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

    fn to_toml_string(&self) -> anyhow::Result<String> {
        toml::to_string_pretty(&VariablesFile {
            variables: self.variables.clone(),
        })
        .context("failed to serialize variables TOML")
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = paths::variables_path();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create variables directory: {}", parent.display())
            })?;
        }

        let content = self.to_toml_string()?;

        fs::write(&path, content)
            .with_context(|| format!("failed to write variables file: {}", path.display()))?;

        Ok(())
    }

    pub fn add_with_mode(&mut self, key: String, value: String, mode: VariableMode) {
        self.variables.insert(key, Variable { value, mode });
    }

    pub fn add(&mut self, key: String, value: String) {
        self.add_with_mode(key, value, VariableMode::Placeholder);
    }

    pub fn delete(&mut self, key: &str) {
        self.variables.remove(key);
    }

    pub fn entries(&self) -> Vec<(&String, &String)> {
        self.variables
            .iter()
            .map(|(key, variable)| (key, &variable.value))
            .collect()
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.variables.get(key).map(|variable| &variable.value)
    }

    pub fn mode(&self, key: &str) -> Option<VariableMode> {
        self.variables.get(key).map(|variable| variable.mode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_substitution_masks_hidden_variables() {
        let mut variables = Variables::default();
        variables.add_with_mode(
            "API_TOKEN".to_string(),
            "secret-token".to_string(),
            VariableMode::Hidden,
        );
        variables.add_with_mode(
            "API_HOST".to_string(),
            "https://api.example.com".to_string(),
            VariableMode::Placeholder,
        );

        let preview = variables.substitute_for_preview("Token={{API_TOKEN}} Host={{API_HOST}}");
        assert_eq!(preview, "Token=hidden Host=https://api.example.com");
    }

    #[test]
    fn execution_substitution_uses_real_values_for_all_modes() {
        let mut variables = Variables::default();
        variables.add_with_mode(
            "API_TOKEN".to_string(),
            "secret-token".to_string(),
            VariableMode::Hidden,
        );
        variables.add_with_mode(
            "API_HOST".to_string(),
            "https://api.example.com".to_string(),
            VariableMode::Placeholder,
        );

        let execution = variables.substitute_for_execution("Token={{API_TOKEN}} Host={{API_HOST}}");
        assert_eq!(execution, "Token=secret-token Host=https://api.example.com");
    }

    #[test]
    fn substitution_keeps_unknown_variables() {
        let variables = Variables::default();

        let preview = variables.substitute_for_preview("{{MISSING}} and {{KNOWN}}");
        let execution = variables.substitute_for_execution("{{MISSING}} and {{KNOWN}}");

        assert_eq!(preview, "{{MISSING}} and {{KNOWN}}");
        assert_eq!(execution, "{{MISSING}} and {{KNOWN}}");
    }

    #[test]
    fn redaction_masks_hidden_values_but_keeps_placeholder_values() {
        let mut variables = Variables::default();
        variables.add_with_mode(
            "LOGIN_EMAIL".to_string(),
            "you@example.com".to_string(),
            VariableMode::Placeholder,
        );
        variables.add_with_mode(
            "LOGIN_PASSWORD".to_string(),
            "super-secret".to_string(),
            VariableMode::Hidden,
        );

        let input = "email=you@example.com password=super-secret";
        let redacted = variables.redact_hidden_values(input);

        assert_eq!(redacted, "email=you@example.com password=hidden");
    }

    #[test]
    fn redaction_masks_longer_hidden_values_first() {
        let mut variables = Variables::default();
        variables.add_with_mode(
            "TOKEN_SUFFIX".to_string(),
            "abc".to_string(),
            VariableMode::Hidden,
        );
        variables.add_with_mode(
            "TOKEN_FULL".to_string(),
            "abc123".to_string(),
            VariableMode::Hidden,
        );

        let redacted = variables.redact_hidden_values("value=abc123");
        assert_eq!(redacted, "value=hidden");
    }

    #[test]
    fn parse_supports_explicit_and_default_modes() {
        let toml_content = r#"
[variables.API_TOKEN]
value = "secret-token"
mode = "hidden"

[variables.API_HOST]
value = "https://api.example.com"
"#;

        let variables = Variables::parse(toml_content).expect("variables should parse");

        assert_eq!(variables.mode("API_TOKEN"), Some(VariableMode::Hidden));
        assert_eq!(variables.mode("API_HOST"), Some(VariableMode::Placeholder));
    }

    #[test]
    fn toml_roundtrip_preserves_values_and_modes() {
        let mut variables = Variables::default();
        variables.add_with_mode(
            "LOGIN_EMAIL".to_string(),
            "you@example.com".to_string(),
            VariableMode::Placeholder,
        );
        variables.add_with_mode(
            "LOGIN_PASSWORD".to_string(),
            "super-secret".to_string(),
            VariableMode::Hidden,
        );

        let serialized = variables
            .to_toml_string()
            .expect("variables should serialize");
        let reparsed = Variables::parse(&serialized).expect("serialized TOML should parse");

        assert_eq!(
            reparsed.substitute_for_execution("{{LOGIN_EMAIL}} {{LOGIN_PASSWORD}}"),
            "you@example.com super-secret"
        );
        assert_eq!(
            reparsed.substitute_for_preview("{{LOGIN_EMAIL}} {{LOGIN_PASSWORD}}"),
            "you@example.com hidden"
        );
    }
}
