use actix_web::{web, App, HttpServer};
use env_logger;
use log::info;

mod errors;
mod models;
mod provider;
mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    info!("Starting oEmbed service on http://localhost:8080");

    let provider = provider::Provider::new();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(provider.clone()))
            .service(routes::oembed_handler)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
