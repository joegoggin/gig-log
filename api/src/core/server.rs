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
        println!("Server running on port 8000");

        HttpServer::new(move || {
            let cors = Cors::default()
                .allowed_origin(&self.env.cors_allowed_origin)
                .allowed_methods(vec!["GET", "PUT", "POST", "DELETE"]);

            App::new()
                .app_data(web::Data::new(self.pool.clone()))
                .wrap(cors)
                .configure(configure_routes)
        })
        .bind(("127.0.0.1", 8000))?
        .run()
        .await
    }
}
