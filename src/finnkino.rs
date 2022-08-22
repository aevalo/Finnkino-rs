use actix_http::error::PayloadError;
use actix_web::{body::BoxBody, http::header::ContentType, HttpRequest, HttpResponse, Responder};
use awc::error::SendRequestError;
use futures::{future, TryFutureExt};
use quick_xml::de::from_str;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct TheatreAreas {
  #[serde(rename(deserialize = "TheatreArea"))]
  pub theatre_areas: std::vec::Vec<TheatreArea>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TheatreArea {
  #[serde(rename(deserialize = "ID"))]
  pub id: String,
  #[serde(rename(deserialize = "Name"))]
  pub name: String,
}

#[derive(Builder, Clone, Debug, Serialize)]
#[builder(setter(into))]
pub struct Error {
  #[builder(setter(strip_option), default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,
  #[builder(setter(strip_option), default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub code: Option<String>,
  #[builder(setter(strip_option), default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub title: Option<String>,
  #[builder(setter(strip_option), default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub detail: Option<String>,
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

pub async fn get_areas() -> Result<std::vec::Vec<TheatreArea>, Error> {
  let areas_xml = get_xml("https://www.finnkino.fi/xml/TheatreAreas").await;
  match areas_xml {
    Err(err) => Err(err),
    Ok(xml) => match from_str::<TheatreAreas>(xml.as_str()) {
      Err(err) => Err(
        ErrorBuilder::default()
          .title("Failed to serialize XML")
          .detail(format!("{:?}", err))
          .build()
          .unwrap(),
      ),
      Ok(areas) => Ok(areas.theatre_areas),
    },
  }
}

async fn get_xml(url: &str) -> Result<String, Error> {
  awc::Client::default()
    .get(url)
    .send()
    .map_err(|err| {
      let mut error_builder = ErrorBuilder::default();
      match err {
        SendRequestError::Url(url_error) => error_builder
          .title("Invalid URL")
          .detail(format!("{:?}", url_error)),
        SendRequestError::Connect(connect_error) => error_builder
          .title("Failed to connect to host")
          .detail(format!("{:?}", connect_error)),
        SendRequestError::Send(send_error) => error_builder
          .title("Error sending request")
          .detail(format!("{:?}", send_error)),
        SendRequestError::Response(parse_error) => error_builder
          .title("Error parsing response")
          .detail(format!("{:?}", parse_error)),
        SendRequestError::Http(http_error) => error_builder
          .title("Http error")
          .detail(format!("{:?}", http_error)),
        SendRequestError::H2(http2_error) => error_builder
          .title("Http2 error")
          .detail(format!("{:?}", http2_error)),
        SendRequestError::Timeout => error_builder
          .title("Response took too long")
          .detail(format!("{:?}", err)),
        SendRequestError::TunnelNotSupported => error_builder
          .title("Tunnels are not supported for HTTP/2 connection")
          .detail(format!("{:?}", err)),
        SendRequestError::Body(body_error) => error_builder
          .title("Error sending request body")
          .detail(format!("{:?}", body_error)),
        SendRequestError::Custom(custom_error, _debug) => error_builder
          .title("Other error")
          .detail(custom_error.to_string()),
        _ => todo!("{}", &err.to_string()),
      };
      error_builder.build().unwrap()
    })
    .and_then(|mut resp| async move {
      if resp.status().is_success() {
        resp
          .body()
          .map_err(|err| {
            let mut error_builder = ErrorBuilder::default();
            match err {
              PayloadError::Incomplete(incomplete_error) => error_builder
                .title("Incomplete")
                .detail(format!("{:?}", incomplete_error)),
              PayloadError::EncodingCorrupted => error_builder
                .title("Encoding corrupted")
                .detail(format!("{:?}", err)),
              PayloadError::Overflow => {
                error_builder.title("Overflow").detail(format!("{:?}", err))
              }
              PayloadError::UnknownLength => error_builder
                .title("Unknown length")
                .detail(format!("{:?}", err)),
              PayloadError::Http2Payload(h2_payload) => error_builder
                .title("Http2 payload error")
                .detail(format!("{:?}", h2_payload)),
              PayloadError::Io(io_error) => error_builder
                .title("IO error")
                .detail(format!("{:?}", io_error)),
              _ => todo!("{}", &err.to_string()),
            };
            error_builder.build().unwrap()
          })
          .and_then(|content| match String::from_utf8(content.to_vec()) {
            Err(err) => {
              let error_builder = ErrorBuilder::default()
                .title("Failed to read XML")
                .detail(format!("{:?}", err))
                .build();
              future::err(error_builder.unwrap())
            }
            Ok(content) => future::ok(content),
          })
          .await
      } else {
        let error_builder = match resp.status().canonical_reason() {
          None => ErrorBuilder::default()
            .status(resp.status().as_str())
            .title("Unknown response status")
            .detail(format!("{:?}", resp.status()))
            .build(),
          Some(reason) => ErrorBuilder::default()
            .status(resp.status().as_str())
            .title(reason)
            .build(),
        };
        Err(error_builder.unwrap())
      }
    })
    .await
}
