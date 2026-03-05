use std::path::Path;

use anyhow::Context;
use tokio::process::Command;

use crate::api_tester::{collection::Route, variables::Variables};

const COOKIE_JAR_PATH: &str = "dev-tools/api-tester/cookies.txt";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurlResponse {
    pub status_code: u16,
    pub headers: Vec<String>,
    pub body: String,
}

pub struct CurlExecutor {
    route: Route,
    variables: Variables,
}

impl CurlExecutor {
    pub fn new(route: Route, variables: Variables) -> Self {
        Self { route, variables }
    }

    pub async fn execute(&self) -> anyhow::Result<CurlResponse> {
        let args = Self::build_args(&self);
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

    fn build_args(&self) -> Vec<String> {
        let cookie_path = Path::new(COOKIE_JAR_PATH);
        let cookie = cookie_path.to_string_lossy().to_string();
        let mut args = vec![
            "-s".to_string(),
            "-X".to_string(),
            self.route.method.to_string(),
        ];

        for header in &self.route.headers {
            args.push("-H".to_string());
            args.push(self.variables.substitute(header));
        }

        if let Some(body) = &self.route.body {
            let body = self.variables.substitute(body);

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
        args.push(self.variables.substitute(&self.route.url));

        args
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
