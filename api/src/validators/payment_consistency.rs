//! Payment request validation helpers.
//!
//! This module validates cross-field constraints for payment create and update
//! payloads so payout-type-specific dates and transfer flags remain consistent.

use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::models::payment::PayoutType;
use crate::routes::payments::{CreatePaymentRequest, UpdatePaymentRequest};

/// Creates a `ValidationError` with a code and message.
fn validation_error(code: &'static str, message: &'static str) -> validator::ValidationError {
    let mut error = validator::ValidationError::new(code);
    error.message = Some(message.into());
    error
}

/// Returns whether a payout type supports transfer tracking fields.
fn payout_type_supports_transfer(payout_type: PayoutType) -> bool {
    matches!(
        payout_type,
        PayoutType::Paypal | PayoutType::Venmo | PayoutType::Zelle
    )
}

/// Validates payout/date/status configuration for a payment payload.
///
/// This private helper enforces total-range checks and transfer-field
/// consistency based on `payout_type`.
///
/// # Arguments
///
/// - `total` - Total payment amount
/// - `payout_type` - Method by which the payment is received
/// - `expected_payout_date` - Optional expected payout date
/// - `expected_transfer_date` - Optional expected transfer date
/// - `transfer_initiated` - Whether transfer has been initiated
/// - `payment_received` - Whether payment has been received
/// - `transfer_received` - Whether transfer has been received
///
/// # Errors
///
/// Returns a `ValidationError` when:
/// - `total` is not greater than 0
/// - non-transfer payout types include transfer-specific fields
/// - transfer flags are set without an expected transfer date
/// - transfer flags violate status ordering dependencies
/// - expected transfer date is earlier than expected payout date
fn validate_payment_configuration(
    total: Decimal,
    payout_type: PayoutType,
    expected_payout_date: Option<NaiveDate>,
    expected_transfer_date: Option<NaiveDate>,
    transfer_initiated: bool,
    payment_received: bool,
    transfer_received: bool,
) -> Result<(), validator::ValidationError> {
    if total <= Decimal::ZERO {
        return Err(validation_error(
            "total_range",
            "Payment total must be greater than 0",
        ));
    }

    let supports_transfer = payout_type_supports_transfer(payout_type);

    if !supports_transfer {
        if expected_transfer_date.is_some() {
            return Err(validation_error(
                "expected_transfer_date_forbidden",
                "Expected transfer date must be omitted for this payout type",
            ));
        }

        if transfer_initiated {
            return Err(validation_error(
                "transfer_initiated_forbidden",
                "Transfer initiated must be false for this payout type",
            ));
        }

        if transfer_received {
            return Err(validation_error(
                "transfer_received_forbidden",
                "Transfer received must be false for this payout type",
            ));
        }
    }

    if supports_transfer {
        if (transfer_initiated || transfer_received) && expected_transfer_date.is_none() {
            return Err(validation_error(
                "expected_transfer_date_required",
                "Expected transfer date is required when transfer status flags are set",
            ));
        }

        if transfer_initiated && !payment_received {
            return Err(validation_error(
                "transfer_initiated_requires_payment_received",
                "Transfer initiated requires payment_received to be true",
            ));
        }

        if transfer_received && !transfer_initiated {
            return Err(validation_error(
                "transfer_received_requires_transfer_initiated",
                "Transfer received requires transfer_initiated to be true",
            ));
        }

        if transfer_received && !payment_received {
            return Err(validation_error(
                "transfer_received_requires_payment_received",
                "Transfer received requires payment_received to be true",
            ));
        }
    }

    if let (Some(payout_date), Some(transfer_date)) = (expected_payout_date, expected_transfer_date)
    {
        if transfer_date < payout_date {
            return Err(validation_error(
                "expected_transfer_date_order",
                "Expected transfer date cannot be earlier than expected payout date",
            ));
        }
    }

    Ok(())
}

/// Validates payout/date/status fields in a create-payment request.
///
/// Used with the `#[validate(custom(...))]` attribute on
/// [`CreatePaymentRequest`].
///
/// See [`create_payment`](crate::routes::payments::handlers::create_payment) for
/// the handler that uses this validation.
pub fn validate_create_payment_consistency(
    request: &CreatePaymentRequest,
) -> Result<(), validator::ValidationError> {
    validate_payment_configuration(
        request.total,
        request.payout_type,
        request.expected_payout_date,
        request.expected_transfer_date,
        request.transfer_initiated,
        request.payment_received,
        request.transfer_received,
    )
}

/// Validates payout/date/status fields in an update-payment request.
///
/// Used with the `#[validate(custom(...))]` attribute on
/// [`UpdatePaymentRequest`].
///
/// See [`update_payment`](crate::routes::payments::handlers::update_payment) for
/// the handler that uses this validation.
pub fn validate_update_payment_consistency(
    request: &UpdatePaymentRequest,
) -> Result<(), validator::ValidationError> {
    validate_payment_configuration(
        request.total,
        request.payout_type,
        request.expected_payout_date,
        request.expected_transfer_date,
        request.transfer_initiated,
        request.payment_received,
        request.transfer_received,
    )
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
    use uuid::Uuid;

    use super::{validate_create_payment_consistency, validate_update_payment_consistency};
    use crate::models::payment::PayoutType;
    use crate::routes::payments::{CreatePaymentRequest, UpdatePaymentRequest};

    fn date(value: &str) -> NaiveDate {
        NaiveDate::parse_from_str(value, "%Y-%m-%d").expect("test date should parse")
    }

    #[test]
    // Verifies totals must be positive.
    fn create_payment_validator_rejects_non_positive_total() {
        let request = CreatePaymentRequest {
            company_id: Uuid::new_v4(),
            total: Decimal::ZERO,
            payout_type: PayoutType::Cash,
            expected_payout_date: None,
            expected_transfer_date: None,
            transfer_initiated: false,
            payment_received: false,
            transfer_received: false,
            tax_withholdings_covered: false,
        };

        let result = validate_create_payment_consistency(&request);
        let error = result.expect_err("validator should reject non-positive totals");

        assert_eq!(error.code.as_ref(), "total_range");
    }

    #[test]
    // Verifies non-transfer payout types reject transfer date and flags.
    fn create_payment_validator_rejects_transfer_fields_for_cash() {
        let request = CreatePaymentRequest {
            company_id: Uuid::new_v4(),
            total: Decimal::new(5000, 2),
            payout_type: PayoutType::Cash,
            expected_payout_date: Some(date("2026-02-01")),
            expected_transfer_date: Some(date("2026-02-02")),
            transfer_initiated: true,
            payment_received: true,
            transfer_received: true,
            tax_withholdings_covered: false,
        };

        let result = validate_create_payment_consistency(&request);
        let error = result.expect_err("validator should reject transfer fields for cash payouts");

        assert_eq!(error.code.as_ref(), "expected_transfer_date_forbidden");
    }

    #[test]
    // Verifies transfer flags require an expected transfer date.
    fn create_payment_validator_requires_expected_transfer_date_when_transfer_flag_is_set() {
        let request = CreatePaymentRequest {
            company_id: Uuid::new_v4(),
            total: Decimal::new(5000, 2),
            payout_type: PayoutType::Paypal,
            expected_payout_date: Some(date("2026-02-01")),
            expected_transfer_date: None,
            transfer_initiated: true,
            payment_received: true,
            transfer_received: false,
            tax_withholdings_covered: false,
        };

        let result = validate_create_payment_consistency(&request);
        let error =
            result.expect_err("validator should require transfer date when transfer flags are set");

        assert_eq!(error.code.as_ref(), "expected_transfer_date_required");
    }

    #[test]
    // Verifies transfer_received requires transfer_initiated.
    fn update_payment_validator_rejects_transfer_received_without_transfer_initiated() {
        let request = UpdatePaymentRequest {
            company_id: Uuid::new_v4(),
            total: Decimal::new(5000, 2),
            payout_type: PayoutType::Venmo,
            expected_payout_date: Some(date("2026-02-01")),
            expected_transfer_date: Some(date("2026-02-02")),
            transfer_initiated: false,
            payment_received: true,
            transfer_received: true,
            tax_withholdings_covered: true,
        };

        let result = validate_update_payment_consistency(&request);
        let error = result.expect_err("validator should enforce transfer status order");

        assert_eq!(
            error.code.as_ref(),
            "transfer_received_requires_transfer_initiated"
        );
    }

    #[test]
    // Verifies expected transfer date cannot be earlier than payout date.
    fn update_payment_validator_rejects_transfer_date_earlier_than_payout_date() {
        let request = UpdatePaymentRequest {
            company_id: Uuid::new_v4(),
            total: Decimal::new(5000, 2),
            payout_type: PayoutType::Zelle,
            expected_payout_date: Some(date("2026-02-10")),
            expected_transfer_date: Some(date("2026-02-09")),
            transfer_initiated: false,
            payment_received: false,
            transfer_received: false,
            tax_withholdings_covered: false,
        };

        let result = validate_update_payment_consistency(&request);
        let error = result.expect_err("validator should reject reverse date ordering");

        assert_eq!(error.code.as_ref(), "expected_transfer_date_order");
    }

    #[test]
    // Verifies valid transfer-capable payouts pass consistency validation.
    fn update_payment_validator_accepts_valid_paypal_configuration() {
        let request = UpdatePaymentRequest {
            company_id: Uuid::new_v4(),
            total: Decimal::new(5000, 2),
            payout_type: PayoutType::Paypal,
            expected_payout_date: Some(date("2026-02-01")),
            expected_transfer_date: Some(date("2026-02-04")),
            transfer_initiated: true,
            payment_received: true,
            transfer_received: false,
            tax_withholdings_covered: true,
        };

        assert!(validate_update_payment_consistency(&request).is_ok());
    }

    #[test]
    // Verifies valid non-transfer payouts pass when transfer fields are clear.
    fn update_payment_validator_accepts_valid_cash_configuration() {
        let request = UpdatePaymentRequest {
            company_id: Uuid::new_v4(),
            total: Decimal::new(12000, 2),
            payout_type: PayoutType::Cash,
            expected_payout_date: Some(date("2026-02-01")),
            expected_transfer_date: None,
            transfer_initiated: false,
            payment_received: true,
            transfer_received: false,
            tax_withholdings_covered: false,
        };

        assert!(validate_update_payment_consistency(&request).is_ok());
    }
}
