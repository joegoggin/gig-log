use api::core::app::App;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let result = App::run().await;

    if let Err(error) = result {
        println!("Error: {}", error)
    }

    Ok(())
}
