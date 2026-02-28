use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PaymentType {
    Hourly,
    Payouts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub company_id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub payment_type: PaymentType,
    pub hourly_rate: Option<f64>,
    pub number_of_payouts: Option<i32>,
    pub payout_amount: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJobRequest {
    pub company_id: Uuid,
    pub title: String,
    pub payment_type: PaymentType,
    pub hourly_rate: Option<f64>,
    pub number_of_payouts: Option<i32>,
    pub payout_amount: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateJobRequest {
    pub title: Option<String>,
    pub payment_type: Option<PaymentType>,
    pub hourly_rate: Option<f64>,
    pub number_of_payouts: Option<i32>,
    pub payout_amount: Option<f64>,
}
