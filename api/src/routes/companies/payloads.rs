//! Request and response payloads for company endpoints.
//!
//! This module contains all data structures used for serializing and
//! deserializing request and response bodies in company handlers.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::company::Company;
use crate::models::payment::PayoutType;
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

/// Company details payload for the company detail page.
///
/// Includes aggregate metrics used by client-side summary cards.
#[derive(Debug, Serialize)]
pub struct CompanyDetails {
    /// Unique identifier for the company.
    pub id: Uuid,
    /// The user who owns this company.
    pub user_id: Uuid,
    /// Company display name.
    pub name: String,
    /// Whether this company requires tax withholding handling.
    pub requires_tax_withholdings: bool,
    /// Tax withholding rate percentage (for example `30.00` for 30%).
    pub tax_withholding_rate: Option<Decimal>,
    /// Timestamp when the company was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Timestamp when the company was last updated.
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Sum of all payment totals associated with this company.
    pub payment_total: Decimal,
    /// Total worked duration across all company work sessions, formatted as `Xh Ym`.
    pub hours: String,
}

/// A lightweight job representation used on the company detail page.
///
/// See [`get_company`](super::handlers::get_company) for the handler that
/// includes this payload.
#[derive(Debug, Serialize)]
pub struct CompanyJobSummary {
    /// Unique identifier for the job.
    pub id: Uuid,
    /// Job title displayed in the company jobs list.
    pub title: String,
}

/// A lightweight payment representation used on the company detail page.
///
/// See [`get_company`](super::handlers::get_company) for the handler that
/// includes this payload.
#[derive(Debug, Serialize)]
pub struct CompanyPaymentSummary {
    /// Unique identifier for the payment.
    pub id: Uuid,
    /// Total payment amount.
    pub total: Decimal,
    /// Payment method for this payout.
    pub payout_type: PayoutType,
    /// Whether the payment has been received.
    pub payment_received: bool,
    /// Whether the transfer has been received.
    pub transfer_received: bool,
}

/// Response body for the company detail route.
///
/// Includes company metadata, aggregate totals, and paginated job/payment lists.
///
/// See [`get_company`](super::handlers::get_company) for the handler that
/// produces this response.
#[derive(Debug, Serialize)]
pub struct CompanyDetailResponse {
    /// Enriched company details including aggregate metrics.
    pub company: CompanyDetails,
    /// Jobs for the requested jobs page.
    pub paginated_jobs: Vec<CompanyJobSummary>,
    /// Whether additional job pages exist after this page.
    pub jobs_has_more: bool,
    /// Payments for the requested payments page.
    pub paginated_payments: Vec<CompanyPaymentSummary>,
    /// Whether additional payment pages exist after this page.
    pub payments_has_more: bool,
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
