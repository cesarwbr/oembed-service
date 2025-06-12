use crate::models::OEmbedRequest;
use crate::provider::Provider;
use actix_web::{get, web, HttpResponse, Responder};

#[get("/oembed")]
async fn oembed_handler(
    query: web::Query<OEmbedRequest>,
    provider: web::Data<Provider>,
) -> impl Responder {
    match provider.get_oembed(query.into_inner()).await {
        Ok(oembed_data) => HttpResponse::Ok().json(oembed_data),
        Err(err) => HttpResponse::BadRequest().body(err.to_string()),
    }
}
