use actix_web::{body::BoxBody, http::header::ContentType, HttpRequest, HttpResponse, Responder};
use quick_xml::de::from_str;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct TheatreAreas {
  #[serde(rename(deserialize = "TheatreArea"))]
  pub theatre_areas: std::vec::Vec<TheatreArea>,
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct TheatreArea {
  #[serde(rename(deserialize = "ID"))]
  pub id: String,
  #[serde(rename(deserialize = "Name"))]
  pub name: String,
}

// Responder
impl Responder for TheatreArea {
  type Body = BoxBody;

  fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
    match serde_json::to_string(&self) {
      Err(error) => HttpResponse::InternalServerError().body(error.to_string()),
      Ok(json) => HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(json),
    }
  }
}

pub async fn get_areas() -> Result<std::vec::Vec<TheatreArea>, String> {
  // create client
  let client = awc::Client::default();

  // construct request
  let req = client.get("https://www.finnkino.fi/xml/TheatreAreas");

  // send request and await response
  match req.send().await {
    Err(err) => Err(format!("{:?}", err)),
    Ok(mut response) => {
      if response.status().is_success() {
        match response.body().await {
          Err(err) => Err(format!("{:?}", err)),
          Ok(body) => match std::str::from_utf8(&body) {
            Err(err) => Err(format!("{:?}", err)),
            Ok(body_str) => match from_str::<TheatreAreas>(body_str) {
              Err(err) => Err(format!("{:?}", err)),
              Ok(areas) => Ok(areas.theatre_areas),
            },
          },
        }
      } else {
        Err(format!("{}", response.status()))
      }
    }
  }
}
