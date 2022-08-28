use super::{Error, ErrorBuilder};

#[allow(dead_code)]
pub async fn get_xml(url: &str) -> Result<String, Error> {
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
