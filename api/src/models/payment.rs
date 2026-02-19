//! Payment model for tracking payments received from companies.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

/// The method by which a payment is received.
#[derive(Debug, Serialize, Deserialize, Type, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "payout_type_enum", rename_all = "lowercase")]
#[allow(dead_code)]
pub enum PayoutType {
    /// Payment via PayPal.
    Paypal,
    /// Cash payment.
    Cash,
    /// Payment by check.
    Check,
    /// Payment via Zelle.
    Zelle,
    /// Payment via Venmo.
    Venmo,
    /// Direct deposit to a bank account.
    #[serde(rename = "direct_deposit")]
    #[sqlx(rename = "direct_deposit")]
    DirectDeposit,
}

/// Represents a payment received from a company for work performed.
///
/// Tracks the payment lifecycle including expected dates, receipt status,
/// and tax withholding coverage.
#[derive(Debug, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct Payment {
    /// Unique identifier for the payment.
    pub id: Uuid,
    /// The user who received this payment.
    pub user_id: Uuid,
    /// The company that made this payment.
    pub company_id: Uuid,
    /// Total payment amount.
    pub total: Decimal,
    /// Method by which the payment was/will be received.
    pub payout_type: PayoutType,
    /// Expected date when the payout will be issued.
    pub expected_payout_date: Option<NaiveDate>,
    /// Expected date when funds will transfer to the user's account.
    pub expected_transfer_date: Option<NaiveDate>,
    /// Whether the transfer has been initiated by the payer.
    pub transfer_initiated: bool,
    /// Whether the payment has been received by the user.
    pub payment_received: bool,
    /// Whether the transferred funds have been received.
    pub transfer_received: bool,
    /// Whether tax withholdings have been accounted for.
    pub tax_withholdings_covered: bool,
    /// Timestamp when the payment record was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the payment record was last updated.
    pub updated_at: DateTime<Utc>,
}
