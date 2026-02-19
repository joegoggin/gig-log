//! Request and response payloads for payment endpoints.
//!
//! This module contains all data structures used for serializing and
//! deserializing request and response bodies in payment handlers.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::payment::{Payment, PayoutType};
use crate::validators::payment_consistency::{
    validate_create_payment_consistency, validate_update_payment_consistency,
};

/// Request body for creating a payment.
///
/// Validates total, payout-type-specific transfer fields, and date/status
/// consistency.
///
/// See [`create_payment`](super::handlers::create_payment) for the handler
/// that processes this request.
#[derive(Debug, Deserialize, Validate)]
#[validate(schema(function = "validate_create_payment_consistency"))]
pub struct CreatePaymentRequest {
    /// Company identifier that owns the payment.
    pub company_id: Uuid,

    /// Total payment amount.
    pub total: Decimal,

    /// Method by which the payment is received.
    pub payout_type: PayoutType,

    /// Expected date when the payout will be issued.
    pub expected_payout_date: Option<NaiveDate>,

    /// Expected date when transferred funds should arrive.
    pub expected_transfer_date: Option<NaiveDate>,

    /// Whether transfer has been initiated.
    pub transfer_initiated: bool,

    /// Whether the payment has been received.
    pub payment_received: bool,

    /// Whether transferred funds have been received.
    pub transfer_received: bool,

    /// Whether tax withholdings are covered for this payment.
    pub tax_withholdings_covered: bool,
}

/// Response body for a single payment.
///
/// See [`create_payment`](super::handlers::create_payment),
/// [`get_payment`](super::handlers::get_payment), and
/// [`update_payment`](super::handlers::update_payment) for handlers that
/// produce this response.
#[derive(Debug, Serialize)]
pub struct PaymentResponse {
    /// Payment resource payload.
    pub payment: Payment,
}

/// Response body for listing payments.
///
/// See [`list_payments`](super::handlers::list_payments) for the handler that
/// produces this response.
#[derive(Debug, Serialize)]
pub struct PaymentsListResponse {
    /// Collection of payments owned by the authenticated user.
    pub payments: Vec<Payment>,
}

/// Request body for updating an existing payment.
///
/// Validates total, payout-type-specific transfer fields, and date/status
/// consistency.
///
/// See [`update_payment`](super::handlers::update_payment) for the handler
/// that processes this request.
#[derive(Debug, Deserialize, Validate)]
#[validate(schema(function = "validate_update_payment_consistency"))]
pub struct UpdatePaymentRequest {
    /// Company identifier that owns the payment.
    pub company_id: Uuid,

    /// Total payment amount.
    pub total: Decimal,

    /// Method by which the payment is received.
    pub payout_type: PayoutType,

    /// Expected date when the payout will be issued.
    pub expected_payout_date: Option<NaiveDate>,

    /// Expected date when transferred funds should arrive.
    pub expected_transfer_date: Option<NaiveDate>,

    /// Whether transfer has been initiated.
    pub transfer_initiated: bool,

    /// Whether the payment has been received.
    pub payment_received: bool,

    /// Whether transferred funds have been received.
    pub transfer_received: bool,

    /// Whether tax withholdings are covered for this payment.
    pub tax_withholdings_covered: bool,
}

/// Response body for deleting a payment.
///
/// See [`delete_payment`](super::handlers::delete_payment) for the handler
/// that produces this response.
#[derive(Debug, Serialize)]
pub struct DeletePaymentResponse {
    /// Human-readable status message.
    pub message: String,
}
