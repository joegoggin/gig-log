use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

use crate::core::{app::AppResult, config::configure_routes, env::Env};

pub struct Server {
    pool: Pool<Postgres>,
    env: Env,
}

impl Server {
    pub async fn new(env: Env) -> AppResult<Server> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&env.database_url)
            .await?;

        Ok(Server { pool, env })
    }

    pub async fn run(self) -> std::io::Result<()> {
        println!("Server running on port {}", self.env.port);

        let env = self.env.clone();

        HttpServer::new(move || {
            let cors = Cors::default()
                .allowed_origin(&env.cors_allowed_origin)
                .allowed_methods(vec!["GET", "PUT", "POST", "DELETE"])
                .allowed_headers(vec!["Content-Type", "Authorization"])
                .supports_credentials();

            App::new()
                .app_data(web::Data::new(self.pool.clone()))
                .app_data(web::Data::new(env.clone()))
                .wrap(cors)
                .configure(configure_routes)
        })
        .bind(("127.0.0.1", self.env.port))?
        .run()
        .await
    }
}
