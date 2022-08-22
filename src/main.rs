#[macro_use]
extern crate derive_builder;

use actix_web::{get, http::header::ContentType, App, HttpResponse, HttpServer, Responder};

mod finnkino;
mod jsonapi;
use finnkino::get_areas;

#[get("/")]
async fn index() -> impl Responder {
  HttpResponse::Ok().body("Hello world!")
}

#[get("/areas")]
async fn areas() -> impl Responder {
  match get_areas().await {
    Err(error) => {
      let errors = jsonapi::Errors::from(error);
      match serde_json::to_string(&errors) {
        Err(error) => HttpResponse::InternalServerError().body(error.to_string()),
        Ok(json) => HttpResponse::Ok()
          .content_type(ContentType::json())
          .body(json),
      }
    }
    Ok(areas) => match serde_json::to_string(&areas) {
      Err(error) => HttpResponse::InternalServerError().body(error.to_string()),
      Ok(json) => HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(json),
    },
  }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  HttpServer::new(|| App::new().service(index).service(areas))
    .workers(4)
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
