use gig_log_api::core::app::App;
use log::error;

#[tokio::main]
async fn main() {
    let result = App::run().await;

    if let Err(error) = result {
        error!("Error: {:#}", error);
    }
}
