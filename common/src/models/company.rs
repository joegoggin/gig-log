use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub requires_tax_withholdings: bool,
    pub tax_withholding_rate: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCompanyRequest {
    pub name: String,
    pub requires_tax_withholdings: bool,
    pub tax_withholding_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCompanyRequest {
    pub name: Option<String>,
    pub requires_tax_withholdings: Option<bool>,
    pub tax_withholding_rate: Option<f64>,
}
