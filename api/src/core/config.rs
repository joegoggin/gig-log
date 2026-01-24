use actix_web::web::ServiceConfig;

use crate::handlers::health::health_check;

pub fn configure_routes(config: &mut ServiceConfig) {
    config.service(health_check);
}
