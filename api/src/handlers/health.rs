use actix_web::{HttpResponse, get};
use serde_json::json;

#[get("/health")]
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "status": "ok"
    }))
}
