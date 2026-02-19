//! Job payment-type request validation helpers.
//!
//! This module validates cross-field constraints for job create and update
//! payloads so payment-type-specific fields remain consistent.

use rust_decimal::Decimal;

use crate::models::job::PaymentType;
use crate::routes::jobs::{CreateJobRequest, UpdateJobRequest};

/// Validates payment configuration for a job payload.
///
/// This private helper enforces required and forbidden fields based on the
/// selected `payment_type`.
///
/// # Arguments
///
/// - `payment_type` - Compensation model (`hourly` or `payouts`)
/// - `number_of_payouts` - Optional number of payouts
/// - `payout_amount` - Optional amount per payout
/// - `hourly_rate` - Optional hourly rate
///
/// # Errors
///
/// Returns a `ValidationError` when:
/// - `hourly` is selected without `hourly_rate`
/// - `hourly_rate` is not greater than 0 for `hourly` jobs
/// - `hourly` is selected with payout fields present
/// - `payouts` is selected without both payout fields
/// - `number_of_payouts` is not greater than 0 for `payouts` jobs
/// - `payout_amount` is not greater than 0 for `payouts` jobs
/// - `payouts` is selected with `hourly_rate` present
fn validate_payment_configuration(
    payment_type: PaymentType,
    number_of_payouts: Option<i32>,
    payout_amount: Option<Decimal>,
    hourly_rate: Option<Decimal>,
) -> Result<(), validator::ValidationError> {
    match payment_type {
        PaymentType::Hourly => {
            let rate = match hourly_rate {
                Some(rate) => rate,
                None => {
                    let mut error = validator::ValidationError::new("hourly_rate_required");
                    error.message =
                        Some("Hourly rate is required when payment type is hourly".into());
                    return Err(error);
                }
            };

            if rate <= Decimal::ZERO {
                let mut error = validator::ValidationError::new("hourly_rate_range");
                error.message = Some("Hourly rate must be greater than 0".into());
                return Err(error);
            }

            if number_of_payouts.is_some() || payout_amount.is_some() {
                let mut error = validator::ValidationError::new("hourly_payout_fields_forbidden");
                error.message =
                    Some("Payout fields must be omitted when payment type is hourly".into());
                return Err(error);
            }
        }
        PaymentType::Payouts => {
            let (payouts, amount) = match (number_of_payouts, payout_amount) {
                (Some(payouts), Some(amount)) => (payouts, amount),
                _ => {
                    let mut error = validator::ValidationError::new("payout_fields_required");
                    error.message = Some(
                        "Number of payouts and payout amount are required when payment type is payouts"
                            .into(),
                    );
                    return Err(error);
                }
            };

            if payouts <= 0 {
                let mut error = validator::ValidationError::new("number_of_payouts_range");
                error.message = Some("Number of payouts must be greater than 0".into());
                return Err(error);
            }

            if amount <= Decimal::ZERO {
                let mut error = validator::ValidationError::new("payout_amount_range");
                error.message = Some("Payout amount must be greater than 0".into());
                return Err(error);
            }

            if hourly_rate.is_some() {
                let mut error = validator::ValidationError::new("payouts_hourly_rate_forbidden");
                error.message =
                    Some("Hourly rate must be omitted when payment type is payouts".into());
                return Err(error);
            }
        }
    }

    Ok(())
}

/// Validates payment fields in a create-job request.
///
/// Used with the `#[validate(custom(...))]` attribute on [`CreateJobRequest`].
///
/// See [`create_job`](crate::routes::jobs::handlers::create_job) for the
/// handler that uses this validation.
pub fn validate_create_job_payment_configuration(
    request: &CreateJobRequest,
) -> Result<(), validator::ValidationError> {
    validate_payment_configuration(
        request.payment_type,
        request.number_of_payouts,
        request.payout_amount,
        request.hourly_rate,
    )
}

/// Validates payment fields in an update-job request.
///
/// Used with the `#[validate(custom(...))]` attribute on [`UpdateJobRequest`].
///
/// See [`update_job`](crate::routes::jobs::handlers::update_job) for the
/// handler that uses this validation.
pub fn validate_update_job_payment_configuration(
    request: &UpdateJobRequest,
) -> Result<(), validator::ValidationError> {
    validate_payment_configuration(
        request.payment_type,
        request.number_of_payouts,
        request.payout_amount,
        request.hourly_rate,
    )
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use uuid::Uuid;

    use super::{
        validate_create_job_payment_configuration, validate_update_job_payment_configuration,
    };
    use crate::models::job::PaymentType;
    use crate::routes::jobs::{CreateJobRequest, UpdateJobRequest};

    #[test]
    // Verifies hourly jobs require an hourly rate.
    fn create_job_validator_rejects_missing_hourly_rate() {
        let request = CreateJobRequest {
            company_id: Uuid::new_v4(),
            title: "Hourly Job".to_string(),
            payment_type: PaymentType::Hourly,
            number_of_payouts: None,
            payout_amount: None,
            hourly_rate: None,
        };

        let result = validate_create_job_payment_configuration(&request);
        let error = result.expect_err("validator should reject missing hourly rate");

        assert_eq!(error.code.as_ref(), "hourly_rate_required");
    }

    #[test]
    // Verifies hourly jobs reject payout-only fields.
    fn create_job_validator_rejects_payout_fields_for_hourly_jobs() {
        let request = CreateJobRequest {
            company_id: Uuid::new_v4(),
            title: "Hourly Job".to_string(),
            payment_type: PaymentType::Hourly,
            number_of_payouts: Some(2),
            payout_amount: Some(Decimal::new(2500, 2)),
            hourly_rate: Some(Decimal::new(3000, 2)),
        };

        let result = validate_create_job_payment_configuration(&request);
        let error = result.expect_err("validator should reject payout fields for hourly jobs");

        assert_eq!(error.code.as_ref(), "hourly_payout_fields_forbidden");
    }

    #[test]
    // Verifies hourly jobs reject non-positive hourly rates.
    fn create_job_validator_rejects_non_positive_hourly_rate() {
        let request = CreateJobRequest {
            company_id: Uuid::new_v4(),
            title: "Hourly Job".to_string(),
            payment_type: PaymentType::Hourly,
            number_of_payouts: None,
            payout_amount: None,
            hourly_rate: Some(Decimal::ZERO),
        };

        let result = validate_create_job_payment_configuration(&request);
        let error = result.expect_err("validator should reject non-positive hourly rate");

        assert_eq!(error.code.as_ref(), "hourly_rate_range");
    }

    #[test]
    // Verifies payout jobs require both payout fields.
    fn update_job_validator_rejects_missing_payout_fields() {
        let request = UpdateJobRequest {
            company_id: Uuid::new_v4(),
            title: "Payout Job".to_string(),
            payment_type: PaymentType::Payouts,
            number_of_payouts: None,
            payout_amount: None,
            hourly_rate: None,
        };

        let result = validate_update_job_payment_configuration(&request);
        let error = result.expect_err("validator should reject missing payout fields");

        assert_eq!(error.code.as_ref(), "payout_fields_required");
    }

    #[test]
    // Verifies payout jobs reject non-positive payout counts.
    fn update_job_validator_rejects_non_positive_number_of_payouts() {
        let request = UpdateJobRequest {
            company_id: Uuid::new_v4(),
            title: "Payout Job".to_string(),
            payment_type: PaymentType::Payouts,
            number_of_payouts: Some(0),
            payout_amount: Some(Decimal::new(5000, 2)),
            hourly_rate: None,
        };

        let result = validate_update_job_payment_configuration(&request);
        let error = result.expect_err("validator should reject non-positive payout counts");

        assert_eq!(error.code.as_ref(), "number_of_payouts_range");
    }

    #[test]
    // Verifies payout jobs reject non-positive payout amounts.
    fn update_job_validator_rejects_non_positive_payout_amount() {
        let request = UpdateJobRequest {
            company_id: Uuid::new_v4(),
            title: "Payout Job".to_string(),
            payment_type: PaymentType::Payouts,
            number_of_payouts: Some(2),
            payout_amount: Some(Decimal::ZERO),
            hourly_rate: None,
        };

        let result = validate_update_job_payment_configuration(&request);
        let error = result.expect_err("validator should reject non-positive payout amounts");

        assert_eq!(error.code.as_ref(), "payout_amount_range");
    }

    #[test]
    // Verifies valid payout jobs pass payment-configuration validation.
    fn update_job_validator_accepts_valid_payout_configuration() {
        let request = UpdateJobRequest {
            company_id: Uuid::new_v4(),
            title: "Payout Job".to_string(),
            payment_type: PaymentType::Payouts,
            number_of_payouts: Some(4),
            payout_amount: Some(Decimal::new(5000, 2)),
            hourly_rate: None,
        };

        assert!(validate_update_job_payment_configuration(&request).is_ok());
    }
}
