mod endpoints;
mod models;
mod services;
mod utils;

use crate::endpoints::sortest_path_endpoint;
use crate::utils::{DEFAULT_LOGGER};

use actix_web::{App, HttpServer};
use log::{info, LevelFilter};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Configure logger before starting the server
    log::set_logger(&DEFAULT_LOGGER).unwrap();
    log::set_max_level(LevelFilter::Trace);

    // Print the server info
    info!("Starting server at 127.0.0.1:8080... Please wait...");

    // Start the server
    HttpServer::new(|| {
        // Print the worker info
        info!("Starting worker for wait new connection");

        // Return the app instance
        App::new().service(sortest_path_endpoint)
    }).bind("127.0.0.1:8080")?.run().await
}
