//! Custom validation functions for request payloads.
//!
//! This module provides validation helpers used with the `validator` crate
//! to perform cross-field validation that cannot be expressed with simple
//! field-level attributes.
//!
//! # Modules
//!
//! - [`company_tax_rate`] - Company tax-withholding cross-field validation
//! - [`password_match`] - Password confirmation validation for sign-up and password reset flows

pub mod company_tax_rate;
pub mod password_match;
