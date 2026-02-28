use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PayoutType {
    Paypal,
    Cash,
    Check,
    Zelle,
    Venmo,
    DirectDeposit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub company_id: Uuid,
    pub total: f64,
    pub payout_type: PayoutType,
    pub expected_payout_date: Option<NaiveDate>,
    pub transfer_initiated: bool,
    pub payment_received: bool,
    pub tax_withholdings_covered: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePaymentRequest {
    pub company_id: Uuid,
    pub total: f64,
    pub payout_type: PayoutType,
    pub expected_payout_date: Option<NaiveDate>,
    pub transfer_initiated: bool,
    pub payment_received: bool,
    pub tax_withholdings_covered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePaymentRequest {
    pub total: Option<f64>,
    pub payout_type: Option<PayoutType>,
    pub expected_payout_date: Option<NaiveDate>,
    pub transfer_initiated: Option<bool>,
    pub payment_received: Option<bool>,
    pub tax_withholdings_covered: Option<bool>,
}
