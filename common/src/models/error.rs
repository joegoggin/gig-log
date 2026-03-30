use serde::{Deserialize, Serialize};

/// A validation error for a specific field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// The field that failed validation, if applicable.
    pub field: Option<String>,
    /// Description of the validation failure.
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

/// A structured API error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    /// HTTP status code for the error.
    pub status_code: u16,
    /// High-level error message.
    pub message: String,
    /// Detailed validation errors, if any.
    pub errors: Option<Vec<ValidationError>>,
}
