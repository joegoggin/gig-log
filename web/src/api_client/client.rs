//! Low-level HTTP client wrapper for frontend API requests.

use gig_log_common::models::error::ApiError;
use reqwest::Client;
use serde::{Serialize, de::DeserializeOwned};

use crate::api_client::error::ClientError;

const BASE_URL: &str = "http://localhost:8000";

/// Wraps a [`reqwest::Client`] for GigLog API requests.
#[derive(Debug, Clone)]
pub struct ApiClient {
    /// Stores the underlying HTTP client.
    client: Client,
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

        Self { client }
    }

    /// Sends a `POST` request and deserializes the response body.
    ///
    /// # Arguments
    ///
    /// * `path` — API path appended to the frontend base URL.
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
        let request = self.client.post(format!("{}{}", BASE_URL, path));
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
    /// * `path` — API path appended to the frontend base URL.
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
        let request = self.client.post(format!("{}{}", BASE_URL, path));
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
    /// * `path` — API path appended to the frontend base URL.
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
        let response = self.client.get(format!("{}{}", BASE_URL, path));

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
}
