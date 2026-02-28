use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub status_code: u16,
    pub message: String,
    pub errors: Option<Vec<ValidationError>>,
}
