use serde::Serialize;
use std::collections::HashMap;
use std::convert::From;
use std::option::Option;
use std::vec::Vec;

use crate::finnkino;

#[derive(Clone, Debug, Serialize)]
pub struct ErrorLink {
  pub about: String,
}

#[derive(Builder, Clone, Debug, Serialize)]
#[builder(setter(into))]
pub struct ErrorSource {
  #[builder(setter(strip_option), default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub pointer: Option<String>,
  #[builder(setter(strip_option), default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub parameter: Option<String>,
}

#[derive(Builder, Clone, Debug, Serialize)]
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

#[derive(Debug, Serialize)]
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
