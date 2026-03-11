use std::fs;
use std::path::Path;

use anyhow::Context;
use tokio::process::Command;

use crate::api_tester::{collection::Route, paths, variables::Variables};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurlResponse {
    pub status_code: u16,
    pub headers: Vec<String>,
    pub body: String,
}

pub struct CurlExecutor {
    route: Route,
    variables: Variables,
    substitute_variables: bool,
}

impl CurlExecutor {
    pub fn new(route: Route, variables: Variables) -> Self {
        Self {
            route,
            variables,
            substitute_variables: true,
        }
    }

    pub fn from_prepared(route: Route) -> Self {
        let mut executor = Self::new(route, Variables::default());
        executor.substitute_variables = false;
        executor
    }

    pub async fn execute(&self) -> anyhow::Result<CurlResponse> {
        let cookie_path = paths::cookie_jar_path();

        if let Some(parent) = cookie_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create cookie directory: {}", parent.display())
            })?;
        }

        let args = Self::build_args(&self, &cookie_path);
        let output = Command::new("curl")
            .args(&args)
            .output()
            .await
            .context("failed to execute curl command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("curl failed: {}", stderr.trim());
        }

        let stdout = String::from_utf8(output.stdout).context("curl output was not valid UTF-8")?;
        Self::parse_response(&stdout)
    }

    fn build_args(&self, cookie_path: &Path) -> Vec<String> {
        let cookie = cookie_path.to_string_lossy().to_string();
        let mut args = vec![
            "-s".to_string(),
            "-X".to_string(),
            self.route.method.to_string(),
        ];

        for header in &self.route.headers {
            args.push("-H".to_string());
            args.push(self.value_for_request(header));
        }

        if let Some(body) = &self.route.body {
            let body = self.value_for_request(body);

            if !body.is_empty() {
                args.push("-d".to_string());
                args.push(body);
            }
        }

        args.push("-b".to_string());
        args.push(cookie.clone());
        args.push("-c".to_string());
        args.push(cookie);
        args.push("-D".to_string());
        args.push("-".to_string());
        args.push("-w".to_string());
        args.push("\n%{http_code}".to_string());
        args.push(self.value_for_request(&self.route.url));

        args
    }

    fn value_for_request(&self, template: &str) -> String {
        if self.substitute_variables {
            self.variables.substitute(template)
        } else {
            template.to_string()
        }
    }

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

    fn split_headers_and_body(raw: &str) -> (&str, &str) {
        if let Some(parts) = raw.rsplit_once("\r\n\r\n") {
            return parts;
        }

        if let Some(parts) = raw.rsplit_once("\n\n") {
            return parts;
        }

        ("", raw)
    }

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
        let args = executor.build_args(Path::new("cookies.txt"));

        assert!(args.contains(&"Authorization: Bearer secret-token".to_string()));
        assert!(args.contains(&"{\"token\":\"secret-token\"}".to_string()));
        assert!(args.contains(&"https://example.com/login".to_string()));
    }

    #[test]
    fn prepared_execution_sends_values_as_is() {
        let executor = CurlExecutor::from_prepared(route_with_template());
        let args = executor.build_args(Path::new("cookies.txt"));

        assert!(args.contains(&"Authorization: Bearer {{TOKEN}}".to_string()));
        assert!(args.contains(&"{\"token\":\"{{TOKEN}}\"}".to_string()));
        assert!(args.contains(&"https://example.com/{{PATH}}".to_string()));
    }
}
