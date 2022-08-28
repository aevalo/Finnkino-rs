use actix_http::error::PayloadError;
use actix_web::{body::BoxBody, http::header::ContentType, HttpRequest, HttpResponse, Responder};
use awc::error::SendRequestError;
use futures::{future, TryFutureExt};
use quick_xml::de::from_str;

use super::{Error, ErrorBuilder, TheatreArea, TheatreAreas};

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

#[allow(dead_code)]
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

#[allow(dead_code)]
async fn get_xml(url: &str) -> Result<String, Error> {
  awc::Client::default()
    .get(url)
    .insert_header(("Accept", "text/xml, application/xml"))
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
              PayloadError::Incomplete(incomplete_error) => {
                if let Some(incomplete) = incomplete_error {
                  error_builder
                    .title("Incomplete")
                    .code(format!("{:?}", incomplete.kind()))
                    .detail(incomplete.to_string())
                } else {
                  error_builder.title("Incomplete")
                }
              }
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
                .title("Failed to parse XML")
                .detail(err.to_string())
                .build();
              future::err(error_builder.unwrap())
            }
            Ok(content_str) => future::ok(content_str),
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

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;
  use std::time::Duration;
  use url::Url;
  use wiremock::matchers::{method, path};
  use wiremock::{Mock, MockServer, ResponseTemplate};

  #[actix_rt::test]
  async fn test_get_xml() {
    // Start a background HTTP server on a random local port
    let mock_server = MockServer::start().await;

    // Arrange the behaviour of the MockServer adding a Mock:
    // when it receives a GET request on '/hello' it will respond with a 200.
    let body = r#"<?xml version="1.0"?>
    <TheatreAreas xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
      <TheatreArea>
        <ID>1029</ID>
        <Name>Valitse alue/teatteri</Name>
      </TheatreArea>
    </TheatreAreas>"#;
    Mock::given(method("GET"))
      .and(path("/xml/TheatreAreas"))
      .respond_with(ResponseTemplate::new(200).set_body_raw(body, "text/xml"))
      // Mounting the mock on the mock server - it's now effective!
      .mount(&mock_server)
      .await;

    let xml_result = get_xml(format!("{}/xml/TheatreAreas", &mock_server.uri()).as_str())
      .await
      .unwrap();

    assert_eq!(xml_result, body);
  }

  #[actix_rt::test]
  async fn test_get_xml_not_found() {
    // Start a background HTTP server on a random local port
    let mock_server = MockServer::start().await;

    // Arrange the behaviour of the MockServer adding a Mock:
    // when it receives a GET request on '/hello' it will respond with a 200.
    Mock::given(method("GET"))
      .and(path("/xml/TheatreAreas"))
      .respond_with(ResponseTemplate::new(200))
      // Mounting the mock on the mock server - it's now effective!
      .mount(&mock_server)
      .await;

    let xml_result = get_xml(format!("{}/xml/TheatreArea", &mock_server.uri()).as_str())
      .await
      .unwrap_err();
    let error = ErrorBuilder::default()
      .status("404")
      .title("Not Found")
      .build()
      .unwrap();

    assert_eq!(xml_result, error);
  }

  #[actix_rt::test]
  async fn test_get_xml_incorrect_scheme() {
    // Start a background HTTP server on a random local port
    let mock_server = MockServer::start().await;
    let mock_url = Url::parse(&mock_server.uri()).unwrap();

    // Arrange the behaviour of the MockServer adding a Mock:
    // when it receives a GET request on '/hello' it will respond with a 200.
    Mock::given(method("GET"))
      .and(path("/xml/TheatreAreas"))
      .respond_with(ResponseTemplate::new(200))
      // Mounting the mock on the mock server - it's now effective!
      .mount(&mock_server)
      .await;

    let xml_result = get_xml(
      format!(
        "htp://{}:{}/xml/TheatreArea",
        mock_url.host().unwrap(),
        mock_url.port().unwrap()
      )
      .as_str(),
    )
    .await
    .unwrap_err();
    let error = ErrorBuilder::default()
      .title("Invalid URL")
      .detail("UnknownScheme")
      .build()
      .unwrap();

    assert_eq!(xml_result, error);
  }

  #[actix_rt::test]
  async fn test_get_xml_timeout() {
    // Start a background HTTP server on a random local port
    let mock_server = MockServer::start().await;
    // Default AWC timeout is 5 seconds
    let delay = Duration::from_secs(6);

    // Arrange the behaviour of the MockServer adding a Mock:
    // when it receives a GET request on '/hello' it will respond with a 200.
    Mock::given(method("GET"))
      .and(path("/xml/TheatreAreas"))
      .respond_with(ResponseTemplate::new(200).set_delay(delay))
      // Mounting the mock on the mock server - it's now effective!
      .mount(&mock_server)
      .await;

    let xml_result = get_xml(format!("{}/xml/TheatreAreas", &mock_server.uri()).as_str())
      .await
      .unwrap_err();
    let error = ErrorBuilder::default()
      .title("Response took too long")
      .detail("Timeout")
      .build()
      .unwrap();

    assert_eq!(xml_result, error);
  }

  #[actix_rt::test]
  async fn test_get_xml_bad_response_body() {
    // Start a background HTTP server on a random local port
    let mock_server = MockServer::start().await;

    // Arrange the behaviour of the MockServer adding a Mock:
    // when it receives a GET request on '/hello' it will respond with a 200.
    let body = r#"<?xml version="1.0"?>
    <TheatreAreas xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
      <TheatreArea>
        <ID>1029</ID>
        <Name>Valitse alue/teatteri</Name>
      </TheatreArea>
    </TheatreAreas>"#;
    Mock::given(method("GET"))
      .and(path("/xml/TheatreAreas"))
      .respond_with(ResponseTemplate::new(200).insert_header("Content-Encoding", "gzip").set_body_raw(body, "text/xml"))
      // Mounting the mock on the mock server - it's now effective!
      .mount(&mock_server)
      .await;

    let xml_result = get_xml(format!("{}/xml/TheatreAreas", &mock_server.uri()).as_str())
      .await
      .unwrap_err();
    let error = ErrorBuilder::default()
      .title("Incomplete")
      .code("InvalidInput")
      .detail("invalid gzip header")
      .build()
      .unwrap();

    assert_eq!(xml_result, error);
  }

  #[actix_rt::test]
  async fn test_get_xml_malformed_xml() {
    // Start a background HTTP server on a random local port
    let mock_server = MockServer::start().await;

    // Arrange the behaviour of the MockServer adding a Mock:
    // when it receives a GET request on '/hello' it will respond with a 200.
    let body = vec![34u8, 228, 166, 164, 110, 237, 166, 164, 44, 34];
    Mock::given(method("GET"))
      .and(path("/xml/TheatreAreas"))
      .respond_with(ResponseTemplate::new(200).insert_header("Content-Type", "text/xml; charset=UTF-8").set_body_bytes(body))
      // Mounting the mock on the mock server - it's now effective!
      .mount(&mock_server)
      .await;

    let xml_result = get_xml(format!("{}/xml/TheatreAreas", &mock_server.uri()).as_str())
      .await
      .unwrap_err();
    let error = ErrorBuilder::default()
      .title("Failed to parse XML")
      .detail("invalid utf-8 sequence of 1 bytes from index 5")
      .build()
      .unwrap();

    assert_eq!(xml_result, error);
  }
}
