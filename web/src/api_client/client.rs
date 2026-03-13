use gig_log_common::models::error::ApiError;
use reqwest::Client;
use serde::{Serialize, de::DeserializeOwned};

use crate::api_client::error::ClientError;

const BASE_URL: &str = "http://localhost:8000";

#[derive(Debug, Clone)]
pub struct ApiClient {
    client: Client,
}

impl ApiClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .build()
            .expect("Failed to build reqwest client");

        Self { client }
    }

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
