use serde::{Deserialize, Serialize};

/// A generic response containing a single message.
#[derive(Serialize, Deserialize)]
pub struct MessageResponse {
    /// The response message.
    pub message: String,
}
