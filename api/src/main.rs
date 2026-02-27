use axum::{Router, routing::get};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));
    let listener = TcpListener::bind("0.0.0.0:8000").await.unwrap();

    println!("Server running on port 8000");
    axum::serve(listener, app).await.unwrap();
}
