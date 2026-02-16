//! Request and response payloads for company endpoints.
//!
//! This module contains all data structures used for serializing and
//! deserializing request and response bodies in company handlers.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::models::company::Company;
use crate::validators::company_tax_rate::{
    validate_create_company_tax_configuration, validate_update_company_tax_configuration,
};

/// Request body for creating a company.
///
/// Validates company naming and tax-withholding configuration.
///
/// See [`create_company`](super::handlers::create_company) for the handler that
/// processes this request.
#[derive(Debug, Deserialize, Validate)]
#[validate(schema(function = "validate_create_company_tax_configuration"))]
pub struct CreateCompanyRequest {
    /// Company display name.
    #[validate(length(min = 1, message = "Company name is required"))]
    pub name: String,

    /// Whether this company requires tax withholding handling.
    pub requires_tax_withholdings: bool,

    /// Tax withholding rate percentage (for example `30.00` for 30%).
    ///
    /// Required when `requires_tax_withholdings` is `true`.
    pub tax_withholding_rate: Option<Decimal>,
}

/// Response body for a single company.
///
/// See [`create_company`](super::handlers::create_company),
/// [`get_company`](super::handlers::get_company), and
/// [`update_company`](super::handlers::update_company) for handlers that
/// produce this response.
#[derive(Debug, Serialize)]
pub struct CompanyResponse {
    /// Company resource payload.
    pub company: Company,
}

/// Response body for listing companies.
///
/// See [`list_companies`](super::handlers::list_companies) for the handler
/// that produces this response.
#[derive(Debug, Serialize)]
pub struct CompaniesListResponse {
    /// Collection of companies owned by the authenticated user.
    pub companies: Vec<Company>,
}

/// Request body for updating an existing company.
///
/// Validates company naming and tax-withholding configuration.
///
/// See [`update_company`](super::handlers::update_company) for the handler
/// that processes this request.
#[derive(Debug, Deserialize, Validate)]
#[validate(schema(function = "validate_update_company_tax_configuration"))]
pub struct UpdateCompanyRequest {
    /// Company display name.
    #[validate(length(min = 1, message = "Company name is required"))]
    pub name: String,

    /// Whether this company requires tax withholding handling.
    pub requires_tax_withholdings: bool,

    /// Tax withholding rate percentage (for example `30.00` for 30%).
    ///
    /// Required when `requires_tax_withholdings` is `true`.
    pub tax_withholding_rate: Option<Decimal>,
}

/// Response body for deleting a company.
///
/// See [`delete_company`](super::handlers::delete_company) for the handler
/// that produces this response.
#[derive(Debug, Serialize)]
pub struct DeleteCompanyResponse {
    /// Human-readable status message.
    pub message: String,
}
