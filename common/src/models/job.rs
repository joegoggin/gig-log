use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// How a job compensates the worker. Serialized as `snake_case`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PaymentType {
    /// Paid based on hours worked at an hourly rate.
    Hourly,
    /// Paid via a fixed number of payouts at a set amount.
    Payouts,
}

/// A job associated with a company.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Unique identifier for the job.
    pub id: Uuid,
    /// The company this job belongs to.
    pub company_id: Uuid,
    /// The user who owns this job record.
    pub user_id: Uuid,
    /// Title or name of the job.
    pub title: String,
    /// How this job compensates the worker.
    pub payment_type: PaymentType,
    /// Hourly rate in dollars. Used when `payment_type` is `Hourly`.
    pub hourly_rate: Option<BigDecimal>,
    /// Total number of payouts. Used when `payment_type` is `Payouts`.
    pub number_of_payouts: Option<i32>,
    /// Amount per payout in dollars. Used when `payment_type` is `Payouts`.
    pub payout_amount: Option<BigDecimal>,
    /// Timestamp when the job was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the job was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Request payload for creating a new job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJobRequest {
    /// The company this job belongs to.
    pub company_id: Uuid,
    /// Title or name of the job.
    pub title: String,
    /// How this job compensates the worker.
    pub payment_type: PaymentType,
    /// Hourly rate in dollars. Used when `payment_type` is `Hourly`.
    pub hourly_rate: Option<BigDecimal>,
    /// Total number of payouts. Used when `payment_type` is `Payouts`.
    pub number_of_payouts: Option<i32>,
    /// Amount per payout in dollars. Used when `payment_type` is `Payouts`.
    pub payout_amount: Option<BigDecimal>,
}

/// Request payload for updating an existing job. All fields are optional.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateJobRequest {
    /// Updated job title.
    pub title: Option<String>,
    /// Updated payment type.
    pub payment_type: Option<PaymentType>,
    /// Updated hourly rate.
    pub hourly_rate: Option<BigDecimal>,
    /// Updated number of payouts.
    pub number_of_payouts: Option<i32>,
    /// Updated payout amount.
    pub payout_amount: Option<BigDecimal>,
}
