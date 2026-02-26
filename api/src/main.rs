use actix_web::{App, HttpResponse, HttpServer, Responder, get};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Server running on port 8000");

    HttpServer::new(|| App::new().service(hello))
        .bind(("127.0.0.1", 8000))?
        .run()
        .await
}
