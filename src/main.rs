use actix_web::{web, App, HttpServer};
use env_logger;
use log::info;

mod errors;
mod firecrawl_service;
mod models;
mod provider;
mod routes;

use routes::oembed_handler;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    info!("Starting oEmbed service on http://localhost:8080");

    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(provider::Provider::new()))
            .service(oembed_handler)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
