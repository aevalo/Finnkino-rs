use quick_xml::de::from_str;
use std::time::Duration;

use libfinnkino_core::finnkino::{Error, ErrorBuilder, TheatreArea, TheatreAreas};

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
  let response = reqwest::Client::new()
    .get(url)
    .timeout(Duration::from_secs(4))
    .send()
    .await;

  match response {
    Err(error) => {
      let error_builder = ErrorBuilder::default()
        .title("Request error")
        .detail(error.to_string())
        .build();
      Err(error_builder.unwrap())
    }
    Ok(resp) => {
      if resp.status().is_success() {
        let content = resp.text().await;
        match content {
          Err(error) => {
            let error_builder = ErrorBuilder::default()
              .title("Response error")
              .detail(error.to_string())
              .build();
            Err(error_builder.unwrap())
          }
          Ok(content_str) => Ok(content_str),
        }
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
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;
  use std::time::Duration;
  use url::Url;
  use wiremock::matchers::{method, path};
  use wiremock::{Mock, MockServer, ResponseTemplate};

  #[rocket::async_test]
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

  #[rocket::async_test]
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

  #[rocket::async_test]
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

    let xml_url = format!(
      "htp://{}:{}/xml/TheatreArea",
      mock_url.host().unwrap(),
      mock_url.port().unwrap()
    );
    let xml_result = get_xml(xml_url.as_str()).await.unwrap_err();
    let error = ErrorBuilder::default()
      .title("Request error")
      .detail(format!(
        "builder error for url ({}): URL scheme is not allowed",
        xml_url
      ))
      .build()
      .unwrap();

    assert_eq!(xml_result, error);
  }

  #[rocket::async_test]
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

    let xml_url = format!("{}/xml/TheatreAreas", &mock_server.uri());
    let xml_result = get_xml(xml_url.as_str()).await.unwrap_err();
    let error = ErrorBuilder::default()
      .title("Request error")
      .detail(format!(
        "error sending request for url ({}): operation timed out",
        xml_url
      ))
      .build()
      .unwrap();

    assert_eq!(xml_result, error);
  }
}
