use gig_log_api::core::app::App;

#[tokio::main]
async fn main() {
    let result = App::run().await;

    if let Err(error) = result {
        println!("Error: {:#?}", error);
    }
}
