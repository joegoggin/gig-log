use actix_web::web::ServiceConfig;

use crate::handlers::auth::{
    confirm_email, current_user, forgot_password, log_in, log_out, set_password, sign_up,
    verify_forgot_password,
};
use crate::handlers::health::health_check;

pub fn configure_routes(config: &mut ServiceConfig) {
    config
        .service(health_check)
        // Auth routes
        .service(sign_up)
        .service(confirm_email)
        .service(log_in)
        .service(log_out)
        .service(current_user)
        .service(forgot_password)
        .service(verify_forgot_password)
        .service(set_password);
}
