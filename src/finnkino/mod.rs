use serde::Serialize;

#[derive(Builder, Clone, Eq, Debug, PartialEq, Serialize)]
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

pub mod actix;
