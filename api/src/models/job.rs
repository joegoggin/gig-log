use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Type, PartialEq, Eq)]
#[sqlx(type_name = "payment_type_enum", rename_all = "lowercase")]
#[allow(dead_code)]
pub enum PaymentType {
    Hourly,
    Payouts,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct Job {
    pub id: Uuid,
    pub company_id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub payment_type: PaymentType,
    pub number_of_payouts: Option<i32>,
    pub payout_amount: Option<Decimal>,
    pub hourly_rate: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
