//! `curl` execution support for API tester requests.
//!
//! This module converts route definitions into temporary curl config files,
//! executes requests asynchronously, and parses responses into structured data
//! for the TUI.

use std::fs;
use std::io::Write;
use std::path::Path;

use anyhow::Context;
use tempfile::NamedTempFile;
use tokio::process::Command;

use crate::api_tester::{collection::Route, paths, variables::Variables};

/// Parsed HTTP response returned by [`CurlExecutor`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurlResponse {
    /// Numeric HTTP status code.
    pub status_code: u16,
    /// Response header lines.
    pub headers: Vec<String>,
    /// Raw response body text.
    pub body: String,
}

/// Executes routes via `curl` and parses the output.
pub struct CurlExecutor {
    /// Route request definition to execute.
    route: Route,
    /// Variable store used for template substitution.
    variables: Variables,
    /// Controls whether templates should be substituted before execution.
    substitute_variables: bool,
}

impl CurlExecutor {
    /// Creates a new executor that substitutes variables before execution.
    ///
    /// # Arguments
    ///
    /// * `route` — Route definition to execute.
    /// * `variables` — Variable store for template substitution.
    ///
    /// # Returns
    ///
    /// A configured [`CurlExecutor`].
    pub fn new(route: Route, variables: Variables) -> Self {
        Self {
            route,
            variables,
            substitute_variables: true,
        }
    }

    /// Creates an executor for already-prepared request values.
    ///
    /// This disables variable substitution so values are sent as-is.
    ///
    /// # Arguments
    ///
    /// * `route` — Prepared route definition.
    ///
    /// # Returns
    ///
    /// A configured [`CurlExecutor`].
    pub fn from_prepared(route: Route) -> Self {
        let mut executor = Self::new(route, Variables::default());
        executor.substitute_variables = false;
        executor
    }

    /// Executes the route through `curl` and parses the HTTP response.
    ///
    /// # Returns
    ///
    /// A parsed [`CurlResponse`] value.
    ///
    /// # Errors
    ///
    /// Returns an [`anyhow::Error`] if curl config generation, command
    /// execution, or response parsing fails.
    pub async fn execute(&self) -> anyhow::Result<CurlResponse> {
        let cookie_path = paths::cookie_jar_path();

        if let Some(parent) = cookie_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create cookie directory: {}", parent.display())
            })?;
        }

        let mut config_file = Self::build_curl_config(&self, &cookie_path)?;
        config_file
            .flush()
            .context("failed to flush curl config file")?;
        let config_path = config_file.path().to_string_lossy().to_string();

        let args = vec!["--config".to_string(), config_path];
        let output = Command::new("curl")
            .args(&args)
            .output()
            .await
            .context("failed to execute curl command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let status = match output.status.code() {
                Some(code) => format!("exit code {code}"),
                None => "terminated by signal".to_string(),
            };

            if stderr.is_empty() {
                anyhow::bail!("curl failed ({status}) with no stderr output");
            }

            anyhow::bail!("curl failed ({status}): {stderr}");
        }

        let stdout = String::from_utf8(output.stdout).context("curl output was not valid UTF-8")?;
        Self::parse_response(&stdout)
    }

    /// Builds a temporary curl configuration file for the request.
    ///
    /// # Arguments
    ///
    /// * `cookie_path` — Cookie jar path used by curl.
    ///
    /// # Returns
    ///
    /// A [`NamedTempFile`] containing curl config directives.
    ///
    /// # Errors
    ///
    /// Returns an [`anyhow::Error`] if config file creation or writing fails.
    fn build_curl_config(&self, cookie_path: &Path) -> anyhow::Result<NamedTempFile> {
        let cookie = cookie_path.to_string_lossy().to_string();
        let mut config = NamedTempFile::new().context("failed to create curl config file")?;

        writeln!(config, "silent")?;
        writeln!(config, "show-error")?;
        writeln!(config, "request = \"{}\"", self.route.method)?;

        for header in &self.route.headers {
            let value = Self::escape_curl_config_value(&self.value_for_request(header));
            writeln!(config, "header = \"{value}\"")?;
        }

        if let Some(body) = &self.route.body {
            let body = self.value_for_request(body);

            if !body.is_empty() {
                let value = Self::escape_curl_config_value(&body);
                writeln!(config, "data = \"{value}\"")?;
            }
        }

        let cookie = Self::escape_curl_config_value(&cookie);
        writeln!(config, "cookie = \"{cookie}\"")?;
        writeln!(config, "cookie-jar = \"{cookie}\"")?;
        writeln!(config, "dump-header = \"-\"")?;
        writeln!(config, "write-out = \"\\n%{{http_code}}\"")?;

        let url = Self::escape_curl_config_value(&self.value_for_request(&self.route.url));
        writeln!(config, "url = \"{url}\"")?;

        Ok(config)
    }

    /// Escapes a value for use in a curl config string literal.
    ///
    /// # Arguments
    ///
    /// * `value` — Raw value to escape.
    ///
    /// # Returns
    ///
    /// An escaped [`String`] safe for curl config output.
    fn escape_curl_config_value(value: &str) -> String {
        value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }

    /// Resolves a request template value for execution.
    ///
    /// # Arguments
    ///
    /// * `template` — Template string that may contain variables.
    ///
    /// # Returns
    ///
    /// A resolved [`String`] value for request execution.
    fn value_for_request(&self, template: &str) -> String {
        if self.substitute_variables {
            self.variables.substitute(template)
        } else {
            template.to_string()
        }
    }

    /// Parses curl stdout into a structured response.
    ///
    /// # Arguments
    ///
    /// * `raw` — Raw curl stdout including dumped headers and status suffix.
    ///
    /// # Returns
    ///
    /// A parsed [`CurlResponse`] value.
    ///
    /// # Errors
    ///
    /// Returns an [`anyhow::Error`] if status suffix or status code parsing
    /// fails.
    fn parse_response(raw: &str) -> anyhow::Result<CurlResponse> {
        let raw = raw.trim_end_matches('\n');

        let (response_with_headers, status_line) = raw
            .rsplit_once('\n')
            .context("missing HTTP status suffix from curl output")?;

        let status_code = status_line
            .trim()
            .parse::<u16>()
            .context("failed to parse HTTP status code")?;

        let (all_headers, body) = Self::split_headers_and_body(response_with_headers);
        let header_block = Self::last_header_block(all_headers);

        let headers = header_block
            .lines()
            .map(|line| line.trim_end_matches('\r'))
            .filter(|line| !line.is_empty())
            .filter(|line| !line.starts_with("HTTP/"))
            .map(ToOwned::to_owned)
            .collect();

        Ok(CurlResponse {
            status_code,
            headers,
            body: body.to_string(),
        })
    }

    /// Splits a combined header/body payload into two slices.
    ///
    /// # Arguments
    ///
    /// * `raw` — Raw response text containing header and body sections.
    ///
    /// # Returns
    ///
    /// A tuple containing `(headers, body)` slices.
    fn split_headers_and_body(raw: &str) -> (&str, &str) {
        if let Some(parts) = raw.rsplit_once("\r\n\r\n") {
            return parts;
        }

        if let Some(parts) = raw.rsplit_once("\n\n") {
            return parts;
        }

        ("", raw)
    }

    /// Selects the final header block from a potentially redirected response.
    ///
    /// # Arguments
    ///
    /// * `headers` — Raw header text that may contain multiple blocks.
    ///
    /// # Returns
    ///
    /// A slice containing the final header block.
    fn last_header_block(headers: &str) -> &str {
        if let Some((_, last)) = headers.rsplit_once("\r\n\r\n") {
            return last;
        }

        if let Some((_, last)) = headers.rsplit_once("\n\n") {
            return last;
        }

        headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api_tester::collection::HttpMethod;

    fn route_with_template() -> Route {
        Route {
            group: "general".to_string(),
            scope_id: "route_test".to_string(),
            name: "test".to_string(),
            method: HttpMethod::Post,
            url: "https://example.com/{{PATH}}".to_string(),
            headers: vec!["Authorization: Bearer {{TOKEN}}".to_string()],
            body: Some("{\"token\":\"{{TOKEN}}\"}".to_string()),
        }
    }

    #[test]
    fn templated_execution_substitutes_variables() {
        let mut variables = Variables::default();
        variables.add_with_mode(
            "PATH".to_string(),
            "login".to_string(),
            crate::api_tester::variables::VariableMode::Placeholder,
        );
        variables.add_with_mode(
            "TOKEN".to_string(),
            "secret-token".to_string(),
            crate::api_tester::variables::VariableMode::Hidden,
        );

        let executor = CurlExecutor::new(route_with_template(), variables);
        let config = executor
            .build_curl_config(Path::new("cookies.txt"))
            .expect("config should build");
        let config_text =
            std::fs::read_to_string(config.path()).expect("config should be readable as text");

        assert!(config_text.contains("header = \"Authorization: Bearer secret-token\""));
        assert!(config_text.contains("data = \"{\\\"token\\\":\\\"secret-token\\\"}\""));
        assert!(config_text.contains("url = \"https://example.com/login\""));
    }

    #[test]
    fn prepared_execution_sends_values_as_is() {
        let executor = CurlExecutor::from_prepared(route_with_template());
        let config = executor
            .build_curl_config(Path::new("cookies.txt"))
            .expect("config should build");
        let config_text =
            std::fs::read_to_string(config.path()).expect("config should be readable as text");

        assert!(config_text.contains("header = \"Authorization: Bearer {{TOKEN}}\""));
        assert!(config_text.contains("data = \"{\\\"token\\\":\\\"{{TOKEN}}\\\"}\""));
        assert!(config_text.contains("url = \"https://example.com/{{PATH}}\""));
    }
}
