//! Job model for representing work assignments associated with companies.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

/// Defines how a job compensates the worker.
#[derive(Debug, Serialize, Deserialize, Type, PartialEq, Eq)]
#[sqlx(type_name = "payment_type_enum", rename_all = "lowercase")]
#[allow(dead_code)]
pub enum PaymentType {
    /// Paid based on hours worked at an hourly rate.
    Hourly,
    /// Paid in fixed amounts (payouts) regardless of time spent.
    Payouts,
}

/// Represents a job or gig that a user performs for a company.
///
/// Jobs track the payment structure (hourly or fixed payouts) and are
/// linked to work sessions for time tracking.
#[derive(Debug, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct Job {
    /// Unique identifier for the job.
    pub id: Uuid,
    /// The company this job is associated with.
    pub company_id: Uuid,
    /// The user who owns/created this job.
    pub user_id: Uuid,
    /// Display name or title of the job.
    pub title: String,
    /// How this job compensates the worker.
    pub payment_type: PaymentType,
    /// Number of fixed payouts for this job. Only applicable when `payment_type` is `Payouts`.
    pub number_of_payouts: Option<i32>,
    /// Amount per payout. Only applicable when `payment_type` is `Payouts`.
    pub payout_amount: Option<Decimal>,
    /// Hourly rate for the job. Only applicable when `payment_type` is `Hourly`.
    pub hourly_rate: Option<Decimal>,
    /// Timestamp when the job was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the job was last updated.
    pub updated_at: DateTime<Utc>,
}
