use api::core::app::App;
use log::error;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let result = App::run().await;

    if let Err(error) = result {
        error!("Error: {}", error);
    }

    Ok(())
}
