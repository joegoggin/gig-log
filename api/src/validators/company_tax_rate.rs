//! Company tax-withholding request validation helpers.
//!
//! This module validates cross-field and range constraints for company create
//! and update payloads.

use rust_decimal::Decimal;

use crate::routes::companies::{CreateCompanyRequest, UpdateCompanyRequest};

/// Validates tax-withholding configuration for a company payload.
///
/// This private helper enforces the expected relationship between
/// `requires_tax_withholdings` and `tax_withholding_rate`.
///
/// # Arguments
///
/// - `requires_tax_withholdings` - Whether withholding is enabled
/// - `tax_withholding_rate` - Optional withholding rate percentage
///
/// # Errors
///
/// Returns a `ValidationError` when:
/// - withholding is enabled without a rate
/// - the rate is outside the `0..=100` percentage range
fn validate_tax_withholding_configuration(
    requires_tax_withholdings: bool,
    tax_withholding_rate: Option<Decimal>,
) -> Result<(), validator::ValidationError> {
    let minimum_rate = Decimal::ZERO;
    let maximum_rate = Decimal::new(100, 0);

    if requires_tax_withholdings {
        let rate = match tax_withholding_rate {
            Some(rate) => rate,
            None => {
                let mut error = validator::ValidationError::new("tax_withholding_rate_required");
                error.message =
                    Some("Tax withholding rate is required when withholdings are enabled".into());
                return Err(error);
            }
        };

        if rate < minimum_rate || rate > maximum_rate {
            let mut error = validator::ValidationError::new("tax_withholding_rate_range");
            error.message = Some("Tax withholding rate must be between 0 and 100".into());
            return Err(error);
        }
    }

    Ok(())
}

/// Validates tax-withholding fields in a create-company request.
///
/// Used with the `#[validate(custom(...))]` attribute on [`CreateCompanyRequest`].
///
/// See [`create_company`](crate::routes::companies::handlers::create_company) for
/// the handler that uses this validation.
pub fn validate_create_company_tax_configuration(
    request: &CreateCompanyRequest,
) -> Result<(), validator::ValidationError> {
    validate_tax_withholding_configuration(
        request.requires_tax_withholdings,
        request.tax_withholding_rate,
    )
}

/// Validates tax-withholding fields in an update-company request.
///
/// Used with the `#[validate(custom(...))]` attribute on [`UpdateCompanyRequest`].
///
/// See [`update_company`](crate::routes::companies::handlers::update_company) for
/// the handler that uses this validation.
pub fn validate_update_company_tax_configuration(
    request: &UpdateCompanyRequest,
) -> Result<(), validator::ValidationError> {
    validate_tax_withholding_configuration(
        request.requires_tax_withholdings,
        request.tax_withholding_rate,
    )
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use super::{
        validate_create_company_tax_configuration, validate_update_company_tax_configuration,
    };
    use crate::routes::companies::{CreateCompanyRequest, UpdateCompanyRequest};

    #[test]
    // Verifies withholding-enabled payloads require a tax rate.
    fn create_company_tax_validator_rejects_missing_rate_when_required() {
        let request = CreateCompanyRequest {
            name: "Acme".to_string(),
            requires_tax_withholdings: true,
            tax_withholding_rate: None,
        };

        let result = validate_create_company_tax_configuration(&request);
        let error = result.expect_err("validator should reject missing tax rate");

        assert_eq!(error.code.as_ref(), "tax_withholding_rate_required");
    }

    #[test]
    // Verifies withholding rates above 100 are rejected.
    fn update_company_tax_validator_rejects_out_of_range_rate() {
        let request = UpdateCompanyRequest {
            name: "Acme".to_string(),
            requires_tax_withholdings: true,
            tax_withholding_rate: Some(Decimal::new(101, 0)),
        };

        let result = validate_update_company_tax_configuration(&request);
        let error = result.expect_err("validator should reject out-of-range rate");

        assert_eq!(error.code.as_ref(), "tax_withholding_rate_range");
    }

    #[test]
    // Verifies withholding-disabled payloads can omit the rate.
    fn create_company_tax_validator_allows_missing_rate_when_not_required() {
        let request = CreateCompanyRequest {
            name: "Acme".to_string(),
            requires_tax_withholdings: false,
            tax_withholding_rate: None,
        };

        assert!(validate_create_company_tax_configuration(&request).is_ok());
    }
}
