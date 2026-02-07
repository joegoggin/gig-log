//! Company model for representing employers or clients that pay users for gigs.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Represents a company that a user works for or receives payment from.
///
/// Companies are associated with a specific user and can optionally track
/// tax withholding requirements for payments received.
#[derive(Debug, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct Company {
    /// Unique identifier for the company.
    pub id: Uuid,
    /// The user who owns/created this company record.
    pub user_id: Uuid,
    /// Display name of the company.
    pub name: String,
    /// Whether this company withholds taxes from payments.
    pub requires_tax_withholdings: bool,
    /// The tax withholding rate as a decimal (e.g., 0.15 for 15%). Only applicable
    /// when `requires_tax_withholdings` is true.
    pub tax_withholding_rate: Option<Decimal>,
    /// Timestamp when the company record was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the company record was last updated.
    pub updated_at: DateTime<Utc>,
}
