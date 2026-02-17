//! API route registration.
//!
//! This module centralizes registration of all HTTP route handlers.

use actix_web::web::ServiceConfig;

use crate::routes::auth::{
    confirm_email, current_user, forgot_password, log_in, log_out, refresh_session, set_password,
    sign_up, verify_forgot_password,
};
use crate::routes::companies::{
    create_company, delete_company, get_company, list_companies, update_company,
};
use crate::routes::health::health_check;

/// Registers all API routes with the Actix service configuration.
///
/// # Arguments
///
/// - `config` - Mutable Actix service configuration used during app startup.
pub fn configure_routes(config: &mut ServiceConfig) {
    config
        // Health routes
        .service(health_check)
        // Auth routes
        .service(sign_up)
        .service(confirm_email)
        .service(log_in)
        .service(log_out)
        .service(refresh_session)
        .service(current_user)
        .service(forgot_password)
        .service(verify_forgot_password)
        .service(set_password)
        // Company routes
        .service(list_companies)
        .service(get_company)
        .service(create_company)
        .service(update_company)
        .service(delete_company);
}
