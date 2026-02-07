//! User model representing authenticated users of the application.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Represents a registered user of the application.
///
/// Users can create companies, jobs, work sessions, and track payments.
/// The password is hashed and excluded from serialization for security.
#[derive(Debug, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct User {
    /// Unique identifier for the user.
    pub id: Uuid,
    /// User's first name.
    pub first_name: String,
    /// User's last name.
    pub last_name: String,
    /// User's email address (used for login).
    pub email: String,
    /// Hashed password (excluded from JSON serialization).
    #[serde(skip_serializing)]
    pub hashed_password: String,
    /// Whether the user has confirmed their email address.
    pub email_confirmed: bool,
    /// Timestamp when the user account was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the user account was last updated.
    pub updated_at: DateTime<Utc>,
}
