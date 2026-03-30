//! Variable storage, substitution, and redaction for API tester routes.
//!
//! This module manages global and route-scoped variables used by request
//! templates. It supports preview-safe rendering, execution substitution, and
//! hidden-value redaction for output surfaces.

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::fs;

use crate::api_tester::paths;

/// Rendering and redaction behavior for a variable value.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum VariableMode {
    /// Uses the value for execution but masks it in previews.
    Hidden,
    /// Displays the value in previews and uses it for execution.
    #[default]
    Placeholder,
}

/// Internal variable representation persisted in TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Variable {
    /// Variable value.
    value: String,
    /// Variable display mode.
    #[serde(default)]
    mode: VariableMode,
}

/// TOML root structure for persisted variables.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct VariablesFile {
    /// Global variables available to every route.
    #[serde(default)]
    global: BTreeMap<String, Variable>,
    /// Route-scoped variable maps keyed by route scope ID.
    #[serde(default)]
    scoped: BTreeMap<String, BTreeMap<String, Variable>>,
}

/// Runtime variable store with global and scoped entries.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Variables {
    /// Global variable map.
    global: BTreeMap<String, Variable>,
    /// Route-scoped variable map.
    scoped: BTreeMap<String, BTreeMap<String, Variable>>,
}

impl Variables {
    /// Loads variables from the default API tester variables file.
    ///
    /// # Returns
    ///
    /// A [`Variables`] instance loaded from disk, or [`Variables::default`]
    /// when the file does not exist.
    ///
    /// # Errors
    ///
    /// Returns an [`anyhow::Error`] if file reading or parsing fails.
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

    /// Parses a TOML variables payload.
    ///
    /// # Arguments
    ///
    /// * `content` — Raw TOML text.
    ///
    /// # Returns
    ///
    /// A parsed [`Variables`] value.
    ///
    /// # Errors
    ///
    /// Returns an [`anyhow::Error`] if TOML parsing fails.
    fn parse(content: &str) -> anyhow::Result<Self> {
        let variables_file: VariablesFile =
            toml::from_str(content).context("failed to parse variables TOML")?;

        Ok(Self {
            global: variables_file.global,
            scoped: variables_file.scoped,
        })
    }

    /// Substitutes variables for request execution.
    ///
    /// # Arguments
    ///
    /// * `template` — Template text containing `{{KEY}}` placeholders.
    ///
    /// # Returns
    ///
    /// A [`String`] with resolved execution values.
    pub fn substitute(&self, template: &str) -> String {
        self.substitute_for_execution(template)
    }

    /// Substitutes global variables for request execution.
    ///
    /// # Arguments
    ///
    /// * `template` — Template text containing `{{KEY}}` placeholders.
    ///
    /// # Returns
    ///
    /// A [`String`] with global substitutions applied.
    pub fn substitute_for_execution(&self, template: &str) -> String {
        self.substitute_with_scope(template, None, |variable| variable.value.as_str())
    }

    /// Substitutes variables for request execution with optional route scope.
    ///
    /// Scoped values override global values when both exist.
    ///
    /// # Arguments
    ///
    /// * `template` — Template text containing `{{KEY}}` placeholders.
    /// * `scope_id` — Optional route scope ID.
    ///
    /// # Returns
    ///
    /// A [`String`] with scoped execution substitutions applied.
    pub fn substitute_for_execution_with_scope(
        &self,
        template: &str,
        scope_id: Option<&str>,
    ) -> String {
        self.substitute_with_scope(template, scope_id, |variable| variable.value.as_str())
    }

    /// Substitutes variables for preview rendering with optional route scope.
    ///
    /// Hidden variables are rendered as `hidden` while placeholder values are
    /// rendered literally.
    ///
    /// # Arguments
    ///
    /// * `template` — Template text containing `{{KEY}}` placeholders.
    /// * `scope_id` — Optional route scope ID.
    ///
    /// # Returns
    ///
    /// A preview-safe [`String`] with hidden values masked.
    pub fn substitute_for_preview_with_scope(
        &self,
        template: &str,
        scope_id: Option<&str>,
    ) -> String {
        self.substitute_with_scope(template, scope_id, |variable| match variable.mode {
            VariableMode::Hidden => "hidden",
            VariableMode::Placeholder => variable.value.as_str(),
        })
    }

    /// Redacts hidden variable values from arbitrary text.
    ///
    /// # Arguments
    ///
    /// * `text` — Input text that may contain resolved secret values.
    /// * `scope_id` — Optional route scope ID used for variable resolution.
    ///
    /// # Returns
    ///
    /// A [`String`] with hidden values replaced by `hidden`.
    pub fn redact_hidden_values_with_scope(&self, text: &str, scope_id: Option<&str>) -> String {
        let mut hidden_values: Vec<&str> = self
            .resolved_variables(scope_id)
            .into_iter()
            .filter_map(|(_, variable)| {
                if variable.mode == VariableMode::Hidden {
                    Some(variable.value.as_str())
                } else {
                    None
                }
            })
            .filter(|value| !value.is_empty())
            .collect();

        hidden_values.sort_by_key(|value| Reverse(value.len()));

        hidden_values
            .into_iter()
            .fold(text.to_string(), |output, hidden_value| {
                output.replace(hidden_value, "hidden")
            })
    }

    /// Substitutes template variables using the provided replacement strategy.
    ///
    /// # Arguments
    ///
    /// * `template` — Template text containing `{{KEY}}` placeholders.
    /// * `scope_id` — Optional route scope ID used for lookup precedence.
    /// * `replacement_for` — Closure that maps a resolved variable to a
    ///   replacement value.
    ///
    /// # Returns
    ///
    /// A [`String`] with resolved substitutions and unknown tokens preserved.
    fn substitute_with_scope<F>(
        &self,
        template: &str,
        scope_id: Option<&str>,
        mut replacement_for: F,
    ) -> String
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

                    if let Some(variable) = self.resolve_variable(key, scope_id) {
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

    /// Resolves a variable by key using scoped-over-global precedence.
    ///
    /// # Arguments
    ///
    /// * `key` — Variable key to resolve.
    /// * `scope_id` — Optional route scope ID.
    ///
    /// # Returns
    ///
    /// An [`Option`] containing a resolved [`Variable`] reference.
    fn resolve_variable<'a>(&'a self, key: &str, scope_id: Option<&str>) -> Option<&'a Variable> {
        if let Some(scope_id) = scope_id
            && let Some(scope_variables) = self.scoped.get(scope_id)
            && let Some(variable) = scope_variables.get(key)
        {
            return Some(variable);
        }

        self.global.get(key)
    }

    /// Builds a merged key map visible for a specific scope.
    ///
    /// Scoped entries override global entries with the same key.
    ///
    /// # Arguments
    ///
    /// * `scope_id` — Optional route scope ID.
    ///
    /// # Returns
    ///
    /// A [`BTreeMap`] of resolved keys and variable references.
    fn resolved_variables<'a>(&'a self, scope_id: Option<&str>) -> BTreeMap<&'a str, &'a Variable> {
        let mut resolved = BTreeMap::new();

        for (key, variable) in &self.global {
            resolved.insert(key.as_str(), variable);
        }

        if let Some(scope_id) = scope_id
            && let Some(scope_variables) = self.scoped.get(scope_id)
        {
            for (key, variable) in scope_variables {
                resolved.insert(key.as_str(), variable);
            }
        }

        resolved
    }

    /// Serializes variables to TOML text.
    ///
    /// # Returns
    ///
    /// A TOML [`String`] representing all global and scoped variables.
    ///
    /// # Errors
    ///
    /// Returns an [`anyhow::Error`] if serialization fails.
    fn to_toml_string(&self) -> anyhow::Result<String> {
        toml::to_string_pretty(&VariablesFile {
            global: self.global.clone(),
            scoped: self.scoped.clone(),
        })
        .context("failed to serialize variables TOML")
    }

    /// Saves variables to the default API tester variables file.
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

    /// Inserts or replaces a global variable with the provided mode.
    ///
    /// # Arguments
    ///
    /// * `key` — Global variable key.
    /// * `value` — Variable value.
    /// * `mode` — Variable rendering mode.
    pub fn add_with_mode(&mut self, key: String, value: String, mode: VariableMode) {
        self.global.insert(key, Variable { value, mode });
    }

    /// Deletes a global variable by key.
    ///
    /// # Arguments
    ///
    /// * `key` — Global variable key.
    pub fn delete(&mut self, key: &str) {
        self.global.remove(key);
    }

    /// Returns all global variable entries.
    ///
    /// # Returns
    ///
    /// A [`Vec`] of global key/value references.
    pub fn entries(&self) -> Vec<(&String, &String)> {
        self.global
            .iter()
            .map(|(key, variable)| (key, &variable.value))
            .collect()
    }

    /// Gets a global variable value by key.
    ///
    /// # Arguments
    ///
    /// * `key` — Global variable key.
    ///
    /// # Returns
    ///
    /// An [`Option`] containing the global variable value reference.
    pub fn get(&self, key: &str) -> Option<&String> {
        self.global.get(key).map(|variable| &variable.value)
    }

    /// Gets a global variable mode by key.
    ///
    /// # Arguments
    ///
    /// * `key` — Global variable key.
    ///
    /// # Returns
    ///
    /// An [`Option`] containing the global variable [`VariableMode`].
    pub fn mode(&self, key: &str) -> Option<VariableMode> {
        self.global.get(key).map(|variable| variable.mode)
    }

    /// Inserts or replaces a scoped variable for a route scope.
    ///
    /// # Arguments
    ///
    /// * `scope_id` — Route scope identifier.
    /// * `key` — Scoped variable key.
    /// * `value` — Scoped variable value.
    /// * `mode` — Scoped variable rendering mode.
    pub fn scoped_add_with_mode(
        &mut self,
        scope_id: impl Into<String>,
        key: String,
        value: String,
        mode: VariableMode,
    ) {
        self.scoped
            .entry(scope_id.into())
            .or_default()
            .insert(key, Variable { value, mode });
    }

    /// Deletes a scoped variable by scope and key.
    ///
    /// # Arguments
    ///
    /// * `scope_id` — Route scope identifier.
    /// * `key` — Scoped variable key.
    pub fn scoped_delete(&mut self, scope_id: &str, key: &str) {
        if let Some(scope_variables) = self.scoped.get_mut(scope_id) {
            scope_variables.remove(key);

            if scope_variables.is_empty() {
                self.scoped.remove(scope_id);
            }
        }
    }

    /// Deletes all scoped variables for a route scope.
    ///
    /// # Arguments
    ///
    /// * `scope_id` — Route scope identifier.
    pub fn delete_scope(&mut self, scope_id: &str) {
        self.scoped.remove(scope_id);
    }

    /// Returns scoped variable entries for a route scope.
    ///
    /// # Arguments
    ///
    /// * `scope_id` — Route scope identifier.
    ///
    /// # Returns
    ///
    /// A [`Vec`] of scoped key/value references.
    pub fn scoped_entries(&self, scope_id: &str) -> Vec<(&String, &String)> {
        self.scoped
            .get(scope_id)
            .map(|scope_variables| {
                scope_variables
                    .iter()
                    .map(|(key, variable)| (key, &variable.value))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Gets a scoped variable value by scope and key.
    ///
    /// # Arguments
    ///
    /// * `scope_id` — Route scope identifier.
    /// * `key` — Scoped variable key.
    ///
    /// # Returns
    ///
    /// An [`Option`] containing the scoped variable value reference.
    pub fn scoped_get(&self, scope_id: &str, key: &str) -> Option<&String> {
        self.scoped
            .get(scope_id)
            .and_then(|scope_variables| scope_variables.get(key))
            .map(|variable| &variable.value)
    }

    /// Gets a scoped variable mode by scope and key.
    ///
    /// # Arguments
    ///
    /// * `scope_id` — Route scope identifier.
    /// * `key` — Scoped variable key.
    ///
    /// # Returns
    ///
    /// An [`Option`] containing the scoped variable [`VariableMode`].
    pub fn scoped_mode(&self, scope_id: &str, key: &str) -> Option<VariableMode> {
        self.scoped
            .get(scope_id)
            .and_then(|scope_variables| scope_variables.get(key))
            .map(|variable| variable.mode)
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

        let preview = variables
            .substitute_for_preview_with_scope("Token={{API_TOKEN}} Host={{API_HOST}}", None);
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

        let preview =
            variables.substitute_for_preview_with_scope("{{MISSING}} and {{KNOWN}}", None);
        let execution = variables.substitute_for_execution("{{MISSING}} and {{KNOWN}}");

        assert_eq!(preview, "{{MISSING}} and {{KNOWN}}");
        assert_eq!(execution, "{{MISSING}} and {{KNOWN}}");
    }

    #[test]
    fn redaction_masks_hidden_values_but_keeps_placeholder_values() {
        let mut variables = Variables::default();
        variables.add_with_mode(
            "EMAIL".to_string(),
            "you@example.com".to_string(),
            VariableMode::Placeholder,
        );
        variables.add_with_mode(
            "LOGIN_PASSWORD".to_string(),
            "super-secret".to_string(),
            VariableMode::Hidden,
        );

        let input = "email=you@example.com password=super-secret";
        let redacted = variables.redact_hidden_values_with_scope(input, None);

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

        let redacted = variables.redact_hidden_values_with_scope("value=abc123", None);
        assert_eq!(redacted, "value=hidden");
    }

    #[test]
    fn scoped_substitution_overrides_global_values() {
        let scope_id = "route_abc123";
        let mut variables = Variables::default();
        variables.add_with_mode(
            "EMAIL".to_string(),
            "global@example.com".to_string(),
            VariableMode::Placeholder,
        );
        variables.scoped_add_with_mode(
            scope_id,
            "EMAIL".to_string(),
            "scoped@example.com".to_string(),
            VariableMode::Placeholder,
        );
        variables.scoped_add_with_mode(
            scope_id,
            "LOGIN_PASSWORD".to_string(),
            "scoped-secret".to_string(),
            VariableMode::Hidden,
        );

        assert_eq!(
            variables.substitute_for_execution_with_scope("{{EMAIL}}", Some(scope_id)),
            "scoped@example.com"
        );
        assert_eq!(
            variables
                .substitute_for_preview_with_scope("{{EMAIL}} {{LOGIN_PASSWORD}}", Some(scope_id),),
            "scoped@example.com hidden"
        );
    }

    #[test]
    fn scoped_redaction_respects_scope_override_mode() {
        let scope_id = "route_abc123";
        let mut variables = Variables::default();
        variables.add_with_mode(
            "TOKEN".to_string(),
            "global-token".to_string(),
            VariableMode::Hidden,
        );
        variables.scoped_add_with_mode(
            scope_id,
            "TOKEN".to_string(),
            "route-token".to_string(),
            VariableMode::Placeholder,
        );

        let redacted =
            variables.redact_hidden_values_with_scope("global-token route-token", Some(scope_id));
        assert_eq!(redacted, "global-token route-token");
    }

    #[test]
    fn parse_supports_global_and_scoped_variables() {
        let toml_content = r#"
[global.EMAIL]
value = "global@example.com"
mode = "placeholder"

[scoped.route_abc123.LOGIN_PASSWORD]
value = "secret-token"
mode = "hidden"

[scoped.route_abc123.EMAIL]
value = "route@example.com"
"#;

        let variables = Variables::parse(toml_content).expect("variables should parse");

        assert_eq!(variables.mode("EMAIL"), Some(VariableMode::Placeholder));
        assert_eq!(
            variables.scoped_mode("route_abc123", "LOGIN_PASSWORD"),
            Some(VariableMode::Hidden)
        );
        assert_eq!(
            variables.scoped_mode("route_abc123", "EMAIL"),
            Some(VariableMode::Placeholder)
        );
    }

    #[test]
    fn toml_roundtrip_preserves_global_and_scoped_variables() {
        let mut variables = Variables::default();
        variables.add_with_mode(
            "EMAIL".to_string(),
            "global@example.com".to_string(),
            VariableMode::Placeholder,
        );
        variables.scoped_add_with_mode(
            "route_abc123",
            "LOGIN_PASSWORD".to_string(),
            "super-secret".to_string(),
            VariableMode::Hidden,
        );

        let serialized = variables
            .to_toml_string()
            .expect("variables should serialize");
        let reparsed = Variables::parse(&serialized).expect("serialized TOML should parse");

        assert_eq!(
            reparsed.substitute_for_execution("{{EMAIL}}"),
            "global@example.com"
        );
        assert_eq!(
            reparsed.substitute_for_execution_with_scope(
                "{{EMAIL}} {{LOGIN_PASSWORD}}",
                Some("route_abc123"),
            ),
            "global@example.com super-secret"
        );
    }

    #[test]
    fn delete_scope_removes_all_scoped_variables_for_route() {
        let mut variables = Variables::default();
        variables.scoped_add_with_mode(
            "route_abc123",
            "TOKEN".to_string(),
            "secret".to_string(),
            VariableMode::Hidden,
        );

        variables.delete_scope("route_abc123");

        assert!(variables.scoped_entries("route_abc123").is_empty());
    }
}
