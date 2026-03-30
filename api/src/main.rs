//! Binary entry point for the GigLog API server.
//!
//! Starts the Tokio runtime and delegates server startup to [`App::run`]. Any
//! startup or runtime failure is logged before the process exits.

use gig_log_api::core::app::App;
use log::error;

/// Starts the GigLog API server process.
///
/// Initializes the async runtime via [`tokio::main`] and invokes [`App::run`]
/// to bootstrap configuration, infrastructure, and HTTP serving.
#[tokio::main]
async fn main() {
    let result = App::run().await;

    if let Err(error) = result {
        error!("Error: {:#}", error);
    }
}
