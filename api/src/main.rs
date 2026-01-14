use actix_cors::Cors;
use actix_web::{App, HttpServer, get, web::Json};
use serde::Serialize;

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
    println!("Server running on port 8000");

    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "PUT", "POST", "DELETE"]);

        App::new().wrap(cors).service(hello)
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}
