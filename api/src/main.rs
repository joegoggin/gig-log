use axum::{Router, routing::get};
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    if auto_apply_migrations_enabled() {
        sqlx::migrate!()
            .run(&pool)
            .await
            .expect("Failed to run migrations");
    }

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .with_state(pool);

    let listener = TcpListener::bind("0.0.0.0:8000").await.unwrap();

    println!("Server running on port 8000");
    axum::serve(listener, app).await.unwrap();
}

fn auto_apply_migrations_enabled() -> bool {
    std::env::var("AUTO_APPLY_MIGRATIONS_ENABLED")
        .ok()
        .and_then(|value| value.parse::<bool>().ok())
        .unwrap_or(true)
}
