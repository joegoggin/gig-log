use serde::{Deserialize, Serialize};

/// A validation error for a specific field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// The field that failed validation, if applicable.
    pub field: Option<String>,
    /// Description of the validation failure.
    pub message: String,
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
