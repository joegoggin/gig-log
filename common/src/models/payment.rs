use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The method used to receive a payment. Serialized as `snake_case`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PayoutType {
    /// Payment via PayPal.
    Paypal,
    /// Payment in cash.
    Cash,
    /// Payment by check.
    Check,
    /// Payment via Zelle.
    Zelle,
    /// Payment via Venmo.
    Venmo,
    /// Payment via direct deposit.
    DirectDeposit,
}

/// A payment received from a company.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    /// Unique identifier for the payment.
    pub id: Uuid,
    /// The user who received this payment.
    pub user_id: Uuid,
    /// The company that issued this payment.
    pub company_id: Uuid,
    /// Total payment amount in dollars.
    pub total: BigDecimal,
    /// Method used to receive the payment.
    pub payout_type: PayoutType,
    /// Expected date the payment will be received.
    pub expected_payout_date: Option<NaiveDate>,
    /// Whether the payment transfer has been initiated by the payer.
    pub transfer_initiated: bool,
    /// Whether the payment has been received by the user.
    pub payment_received: bool,
    /// Whether tax withholdings have been accounted for.
    pub tax_withholdings_covered: bool,
    /// Timestamp when the payment record was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the payment record was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Request payload for creating a new payment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePaymentRequest {
    /// The company that issued this payment.
    pub company_id: Uuid,
    /// Total payment amount in dollars.
    pub total: BigDecimal,
    /// Method used to receive the payment.
    pub payout_type: PayoutType,
    /// Expected date the payment will be received.
    pub expected_payout_date: Option<NaiveDate>,
    /// Whether the payment transfer has been initiated by the payer.
    pub transfer_initiated: bool,
    /// Whether the payment has been received by the user.
    pub payment_received: bool,
    /// Whether tax withholdings have been accounted for.
    pub tax_withholdings_covered: bool,
}

/// Request payload for updating an existing payment. All fields are optional.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePaymentRequest {
    /// Updated total payment amount.
    pub total: Option<BigDecimal>,
    /// Updated payout method.
    pub payout_type: Option<PayoutType>,
    /// Updated expected payout date.
    pub expected_payout_date: Option<NaiveDate>,
    /// Updated transfer initiation status.
    pub transfer_initiated: Option<bool>,
    /// Updated payment received status.
    pub payment_received: Option<bool>,
    /// Updated tax withholdings status.
    pub tax_withholdings_covered: Option<bool>,
}
