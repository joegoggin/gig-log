use actix_cors::Cors;
use actix_web::{get, web, App, HttpServer, Responder};
use dotenvy::dotenv;
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use std::env;

mod models;

#[derive(Serialize)]
struct Message {
    message: String,
}

#[get("/")]
async fn hello() -> impl Responder {
    web::Json(Message {
        message: "Hello from api!".to_string(),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let allowed_origin =
        env::var("CORS_ALLOWED_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool.");

    println!("Server running on port 8000");
    println!("CORS allowed origin: {}", allowed_origin);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(&allowed_origin)
            .allowed_methods(vec!["GET", "PUT", "POST", "DELETE"]);

        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(cors)
            .service(hello)
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}
