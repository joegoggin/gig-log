use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Type, PartialEq, Eq)]
#[sqlx(type_name = "payout_type_enum", rename_all = "lowercase")] // check lowercase vs snake_case if values have underscores
pub enum PayoutType {
    Paypal,
    Cash,
    Check,
    Zelle,
    Venmo,
    #[serde(rename = "direct_deposit")]
    #[sqlx(rename = "direct_deposit")]
    DirectDeposit,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Payment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub company_id: Uuid,
    pub total: Decimal,
    pub payout_type: PayoutType,
    pub expected_payout_date: Option<NaiveDate>,
    pub expected_transfer_date: Option<NaiveDate>,
    pub transfer_initiated: bool,
    pub payment_received: bool,
    pub transfer_received: bool,
    pub tax_withholdings_covered: bool,
}
