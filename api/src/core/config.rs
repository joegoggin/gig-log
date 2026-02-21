//! API route registration.
//!
//! This module centralizes registration of all HTTP route handlers.

use actix_web::web::ServiceConfig;

use crate::routes::appearance::{create_custom_palette, get_appearance, set_active_palette};
use crate::routes::auth::{
    change_password, confirm_email, confirm_email_change, current_user, forgot_password, log_in,
    log_out, refresh_session, request_email_change, set_password, sign_up, verify_forgot_password,
};
use crate::routes::companies::{
    create_company, delete_company, get_company, list_companies, update_company,
};
use crate::routes::health::health_check;
use crate::routes::jobs::{create_job, delete_job, get_job, list_jobs, update_job};
use crate::routes::payments::{
    create_payment, delete_payment, get_payment, list_payments, update_payment,
};
use crate::routes::work_sessions::{
    complete_work_session, get_active_work_session, list_work_sessions_for_job, pause_work_session,
    resume_work_session, start_work_session,
};

/// Registers all API routes with the Actix service configuration.
///
/// # Arguments
///
/// - `config` - Mutable Actix service configuration used during app startup.
pub fn configure_routes(config: &mut ServiceConfig) {
    config
        // Health routes
        .service(health_check)
        // Appearance routes
        .service(get_appearance)
        .service(create_custom_palette)
        .service(set_active_palette)
        // Auth routes
        .service(sign_up)
        .service(confirm_email)
        .service(log_in)
        .service(log_out)
        .service(refresh_session)
        .service(current_user)
        .service(request_email_change)
        .service(confirm_email_change)
        .service(forgot_password)
        .service(verify_forgot_password)
        .service(set_password)
        .service(change_password)
        // Company routes
        .service(list_companies)
        .service(get_company)
        .service(create_company)
        .service(update_company)
        .service(delete_company)
        // Job routes
        .service(list_jobs)
        .service(get_job)
        .service(create_job)
        .service(update_job)
        .service(delete_job)
        // Payment routes
        .service(list_payments)
        .service(get_payment)
        .service(create_payment)
        .service(update_payment)
        .service(delete_payment)
        // Work session routes
        .service(start_work_session)
        .service(pause_work_session)
        .service(resume_work_session)
        .service(complete_work_session)
        .service(get_active_work_session)
        .service(list_work_sessions_for_job);
}
