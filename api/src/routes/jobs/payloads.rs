//! Request and response payloads for job endpoints.
//!
//! This module contains all data structures used for serializing and
//! deserializing request and response bodies in job handlers.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::job::{Job, PaymentType};
use crate::validators::job_payment_type::{
    validate_create_job_payment_configuration, validate_update_job_payment_configuration,
};

/// Request body for creating a job.
///
/// Validates payment-type-specific field requirements and exclusions.
///
/// See [`create_job`](super::handlers::create_job) for the handler that
/// processes this request.
#[derive(Debug, Deserialize, Validate)]
#[validate(schema(function = "validate_create_job_payment_configuration"))]
pub struct CreateJobRequest {
    /// Company identifier that owns the job.
    pub company_id: Uuid,

    /// Job display title.
    #[validate(length(min = 1, message = "Job title is required"))]
    pub title: String,

    /// Compensation model for the job.
    pub payment_type: PaymentType,

    /// Number of fixed payouts. Required when `payment_type` is `payouts`.
    pub number_of_payouts: Option<i32>,

    /// Amount paid per payout. Required when `payment_type` is `payouts`.
    pub payout_amount: Option<Decimal>,

    /// Hourly rate. Required when `payment_type` is `hourly`.
    pub hourly_rate: Option<Decimal>,
}

/// Response body for a single job.
///
/// See [`create_job`](super::handlers::create_job),
/// [`get_job`](super::handlers::get_job), and
/// [`update_job`](super::handlers::update_job) for handlers that produce this
/// response.
#[derive(Debug, Serialize)]
pub struct JobResponse {
    /// Job resource payload.
    pub job: Job,
}

/// Response body for listing jobs.
///
/// See [`list_jobs`](super::handlers::list_jobs) for the handler that produces
/// this response.
#[derive(Debug, Serialize)]
pub struct JobsListResponse {
    /// Collection of jobs owned by the authenticated user.
    pub jobs: Vec<Job>,
}

/// Request body for updating an existing job.
///
/// Validates payment-type-specific field requirements and exclusions.
///
/// See [`update_job`](super::handlers::update_job) for the handler that
/// processes this request.
#[derive(Debug, Deserialize, Validate)]
#[validate(schema(function = "validate_update_job_payment_configuration"))]
pub struct UpdateJobRequest {
    /// Company identifier that owns the job.
    pub company_id: Uuid,

    /// Job display title.
    #[validate(length(min = 1, message = "Job title is required"))]
    pub title: String,

    /// Compensation model for the job.
    pub payment_type: PaymentType,

    /// Number of fixed payouts. Required when `payment_type` is `payouts`.
    pub number_of_payouts: Option<i32>,

    /// Amount paid per payout. Required when `payment_type` is `payouts`.
    pub payout_amount: Option<Decimal>,

    /// Hourly rate. Required when `payment_type` is `hourly`.
    pub hourly_rate: Option<Decimal>,
}

/// Response body for deleting a job.
///
/// See [`delete_job`](super::handlers::delete_job) for the handler that
/// produces this response.
#[derive(Debug, Serialize)]
pub struct DeleteJobResponse {
    /// Human-readable status message.
    pub message: String,
}
