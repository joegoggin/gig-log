use actix_cors::Cors;
use actix_web::{App, HttpServer, get, web::Json};
use serde::Serialize;
use std::env;

#[derive(Serialize)]
struct Message {
    message: String,
}

#[get("/")]
async fn hello() -> Json<Message> {
    Json(Message {
        message: "Hello from api!".to_string(),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let allowed_origin =
        env::var("CORS_ALLOWED_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".to_string());

    println!("Server running on port 8000");
    println!("CORS allowed origin: {}", allowed_origin);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(&allowed_origin)
            .allowed_methods(vec!["GET", "PUT", "POST", "DELETE"]);

        App::new().wrap(cors).service(hello)
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}
