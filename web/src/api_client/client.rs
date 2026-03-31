//! Low-level HTTP client wrapper for frontend API requests.

use gig_log_common::models::error::ApiError;
use reqwest::Client;
use serde::{Serialize, de::DeserializeOwned};

use crate::api_client::error::ClientError;

const DEFAULT_BASE_URL: &str = "http://localhost:8000";
#[cfg(target_arch = "wasm32")]
const DEFAULT_API_PORT: u16 = 8000;

/// Wraps a [`reqwest::Client`] for GigLog API requests.
#[derive(Debug, Clone)]
pub struct ApiClient {
    /// Stores the underlying HTTP client.
    client: Client,
    /// Stores the resolved API base URL for all requests.
    base_url: String,
}

impl ApiClient {
    /// Creates a new [`ApiClient`].
    ///
    /// # Returns
    ///
    /// An initialized [`ApiClient`].
    pub fn new() -> Self {
        let client = Client::builder()
            .build()
            .expect("Failed to build reqwest client");
        let base_url = Self::resolve_base_url();

        Self { client, base_url }
    }

    /// Sends a `POST` request and deserializes the response body.
    ///
    /// # Arguments
    ///
    /// * `path` — API path appended to the resolved API base URL.
    /// * `body` — Optional serializable request payload.
    ///
    /// # Returns
    ///
    /// A [`Result`] containing the deserialized response payload on success.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] if request execution fails, response
    /// deserialization fails, or the API returns an error payload.
    pub async fn post<T: Serialize, R: DeserializeOwned>(
        &self,
        path: &str,
        body: Option<&T>,
    ) -> Result<R, ClientError> {
        let request = self.client.post(self.build_url(path));
        let request = Self::with_credentials(request);

        let response = match body {
            Some(b) => request.json(b).send().await,
            None => request.send().await,
        }
        .map_err(|e| ClientError::Network(e.to_string()))?;

        if response.status().is_success() {
            response
                .json::<R>()
                .await
                .map_err(|e| ClientError::Network(e.to_string()))
        } else {
            let api_error = response
                .json::<ApiError>()
                .await
                .map_err(|e| ClientError::Network(e.to_string()))?;
            Err(ClientError::Api(api_error))
        }
    }

    /// Sends a `POST` request that expects no response payload.
    ///
    /// # Arguments
    ///
    /// * `path` — API path appended to the resolved API base URL.
    /// * `body` — Optional serializable request payload.
    ///
    /// # Returns
    ///
    /// A [`Result`] containing `()` on success.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] if request execution fails or the API returns
    /// an error payload.
    pub async fn post_no_content<T: Serialize>(
        &self,
        path: &str,
        body: Option<&T>,
    ) -> Result<(), ClientError> {
        let request = self.client.post(self.build_url(path));
        let request = Self::with_credentials(request);

        let response = match body {
            Some(b) => request.json(b).send().await,
            None => request.send().await,
        }
        .map_err(|e| ClientError::Network(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let api_error = response
                .json::<ApiError>()
                .await
                .map_err(|e| ClientError::Network(e.to_string()))?;
            Err(ClientError::Api(api_error))
        }
    }

    /// Sends a `GET` request and deserializes the response body.
    ///
    /// # Arguments
    ///
    /// * `path` — API path appended to the resolved API base URL.
    ///
    /// # Returns
    ///
    /// A [`Result`] containing the deserialized response payload on success.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] if request execution fails, response
    /// deserialization fails, or the API returns an error payload.
    pub async fn get<R: DeserializeOwned>(&self, path: &str) -> Result<R, ClientError> {
        let response = self.client.get(self.build_url(path));

        let response = Self::with_credentials(response)
            .send()
            .await
            .map_err(|e| ClientError::Network(e.to_string()))?;

        if response.status().is_success() {
            response
                .json::<R>()
                .await
                .map_err(|e| ClientError::Network(e.to_string()))
        } else {
            let api_error = response
                .json::<ApiError>()
                .await
                .map_err(|e| ClientError::Network(e.to_string()))?;
            Err(ClientError::Api(api_error))
        }
    }

    /// Applies browser credential settings to a request builder.
    ///
    /// # Arguments
    ///
    /// * `builder` — Request builder to update.
    ///
    /// # Returns
    ///
    /// The request builder configured with credential behavior for the target
    /// architecture.
    fn with_credentials(builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        #[cfg(target_arch = "wasm32")]
        {
            builder.fetch_credentials_include()
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            builder
        }
    }

    /// Builds an absolute request URL by joining the base URL and API path.
    ///
    /// # Arguments
    ///
    /// * `path` — API path appended to the resolved base URL.
    ///
    /// # Returns
    ///
    /// A request URL string ready for reqwest.
    fn build_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// Resolves the API base URL used by the frontend client.
    ///
    /// Prefers a compile-time `WEB_API_BASE_URL` override. In the browser,
    /// falls back to the current hostname on port `8000` so local and Tailnet
    /// access both target the correct API host. Defaults to localhost when no
    /// browser hostname is available.
    ///
    /// # Returns
    ///
    /// A normalized API base URL.
    fn resolve_base_url() -> String {
        if let Some(configured_base_url) =
            option_env!("WEB_API_BASE_URL").and_then(Self::normalize_base_url)
        {
            return configured_base_url;
        }

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(browser_base_url) = Self::browser_base_url() {
                return browser_base_url;
            }
        }

        DEFAULT_BASE_URL.to_string()
    }

    /// Builds a browser-derived API base URL from the current hostname.
    ///
    /// # Returns
    ///
    /// An optional API base URL using the current hostname on port `8000`.
    #[cfg(target_arch = "wasm32")]
    fn browser_base_url() -> Option<String> {
        let window = web_sys::window()?;
        let hostname = window.location().hostname().ok()?;
        let hostname = hostname.trim();

        if hostname.is_empty() {
            return None;
        }

        Some(format!("http://{}:{}", hostname, DEFAULT_API_PORT))
    }

    /// Normalizes a base URL string for request construction.
    ///
    /// # Arguments
    ///
    /// * `base_url` — Raw base URL value from configuration.
    ///
    /// # Returns
    ///
    /// An optional normalized base URL without trailing `/`.
    fn normalize_base_url(base_url: &str) -> Option<String> {
        let normalized = base_url.trim().trim_end_matches('/');

        if normalized.is_empty() {
            None
        } else {
            Some(normalized.to_string())
        }
    }
}
