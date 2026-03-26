use gig_log_common::models::error::ApiError;

#[derive(Debug, Clone)]
pub enum ClientError {
    Api(ApiError),
    Network(String),
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::Api(err) => write!(f, "{}", err.message),
            ClientError::Network(msg) => write!(f, "Network error: {}", msg),
        }
    }
}
