use quick_xml::de::from_str;

use super::{Error, ErrorBuilder, TheatreArea, TheatreAreas};

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
  let response = reqwest::Client::new().get(url).send().await;

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
