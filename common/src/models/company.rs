use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A company that a user works for.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    /// Unique identifier for the company.
    pub id: Uuid,
    /// The user who owns this company record.
    pub user_id: Uuid,
    /// Name of the company.
    pub name: String,
    /// Whether this company requires tax withholdings on payments.
    pub requires_tax_withholdings: bool,
    /// Tax withholding rate as a decimal (e.g., 0.15 for 15%). Only applicable
    /// when `requires_tax_withholdings` is true.
    pub tax_withholding_rate: Option<f64>,
    /// Timestamp when the company was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the company was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Request payload for creating a new company.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCompanyRequest {
    /// Name of the company.
    pub name: String,
    /// Whether this company requires tax withholdings on payments.
    pub requires_tax_withholdings: bool,
    /// Optional tax withholding rate as a decimal.
    pub tax_withholding_rate: Option<f64>,
}

/// Request payload for updating an existing company. All fields are optional.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCompanyRequest {
    /// Updated company name.
    pub name: Option<String>,
    /// Updated tax withholding requirement.
    pub requires_tax_withholdings: Option<bool>,
    /// Updated tax withholding rate.
    pub tax_withholding_rate: Option<f64>,
}
