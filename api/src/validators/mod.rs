//! Custom validation functions for request payloads.
//!
//! This module provides validation helpers used with the `validator` crate
//! to perform cross-field validation that cannot be expressed with simple
//! field-level attributes.
//!
//! # Modules
//!
//! - [`company_tax_rate`] - Company tax-withholding cross-field validation
//! - [`job_payment_type`] - Job payment-type cross-field validation
//! - [`password_match`] - Password confirmation validation for sign-up and password-update flows
//! - [`payment_consistency`] - Payment payout/date/status cross-field validation

pub mod company_tax_rate;
pub mod job_payment_type;
pub mod password_match;
pub mod payment_consistency;
