use std::collections::HashMap;
use std::convert::From;
use std::option::Option;
use std::vec::Vec;

use serde::Serialize;

use crate::finnkino;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ErrorLink {
  pub about: String,
}

#[derive(Builder, Clone, Debug, PartialEq, Serialize)]
#[builder(setter(into))]
pub struct ErrorSource {
  #[builder(setter(strip_option), default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub pointer: Option<String>,
  #[builder(setter(strip_option), default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub parameter: Option<String>,
}

#[derive(Builder, Clone, Debug, PartialEq, Serialize)]
#[builder(setter(into))]
pub struct Error {
  #[builder(setter(strip_option), default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[builder(setter(strip_option), default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub links: Option<ErrorLink>,
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
  #[builder(setter(strip_option), default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub source: Option<ErrorSource>,
  #[builder(setter(strip_option), default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub meta: Option<HashMap<String, String>>,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Errors {
  pub errors: Vec<Error>,
}

impl From<finnkino::Error> for Errors {
  fn from(finnkino_error: finnkino::Error) -> Self {
    let mut error_builder = ErrorBuilder::default();
    if let Some(status) = &finnkino_error.status {
      error_builder.status(status);
    }
    if let Some(code) = &finnkino_error.code {
      error_builder.code(code);
    }
    if let Some(title) = &finnkino_error.title {
      error_builder.title(title);
    }
    if let Some(detail) = &finnkino_error.detail {
      error_builder.detail(detail);
    }
    Errors {
      errors: vec![error_builder.build().unwrap()],
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;

  #[test]
  fn test_can_convert_from_finnkino_error_with_status() {
    let finnkino_error = finnkino::ErrorBuilder::default().status("some-status").build().unwrap();
    let json_errors = Errors::from(finnkino_error);
    let expected_errors = Errors {
      errors: vec![ErrorBuilder::default().status("some-status").build().unwrap()]
    };

    assert_eq!(expected_errors, json_errors);
  }

  #[test]
  fn test_can_convert_from_finnkino_error_with_code() {
    let finnkino_error = finnkino::ErrorBuilder::default().code("some-code").build().unwrap();
    let json_errors = Errors::from(finnkino_error);
    let expected_errors = Errors {
      errors: vec![ErrorBuilder::default().code("some-code").build().unwrap()]
    };

    assert_eq!(expected_errors, json_errors);
  }

  #[test]
  fn test_can_convert_from_finnkino_error_with_title() {
    let finnkino_error = finnkino::ErrorBuilder::default().title("some-title").build().unwrap();
    let json_errors = Errors::from(finnkino_error);
    let expected_errors = Errors {
      errors: vec![ErrorBuilder::default().title("some-title").build().unwrap()]
    };

    assert_eq!(expected_errors, json_errors);
  }

  #[test]
  fn test_can_convert_from_finnkino_error_with_detail() {
    let finnkino_error = finnkino::ErrorBuilder::default().detail("some-detail").build().unwrap();
    let json_errors = Errors::from(finnkino_error);
    let expected_errors = Errors {
      errors: vec![ErrorBuilder::default().detail("some-detail").build().unwrap()]
    };

    assert_eq!(expected_errors, json_errors);
  }

  #[test]
  fn test_can_convert_from_finnkino_error_all_fields() {
    let finnkino_error = finnkino::ErrorBuilder::default()
      .status("some-status")
      .code("some-code")
      .title("some-title")
      .detail("some-detail")
      .build()
      .unwrap();
    let json_errors = Errors::from(finnkino_error);
    let expected_error = ErrorBuilder::default()
      .status("some-status")
      .code("some-code")
      .title("some-title")
      .detail("some-detail")
      .build()
      .unwrap();
    let expected_errors = Errors {
      errors: vec![expected_error]
    };

    assert_eq!(expected_errors, json_errors);
  }
}
