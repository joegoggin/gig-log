use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: Option<String>,
    pub message: String,
}

impl Default for ValidationError {
    fn default() -> Self {
        Self {
            field: None,
            message: "".to_string(),
        }
    }
}

impl ValidationError {
    pub fn new(field: Option<String>, message: impl Into<String>) -> Self {
        Self {
            field: field,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub status_code: u16,
    pub message: String,
    pub errors: Option<Vec<ValidationError>>,
}
