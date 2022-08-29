use serde::{Deserialize, Serialize};
use std::vec::Vec;

#[derive(Deserialize, Debug)]
pub struct TheatreAreas {
  #[serde(rename(deserialize = "TheatreArea"))]
  pub theatre_areas: Vec<TheatreArea>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct TheatreArea {
  #[serde(rename(deserialize = "ID"))]
  pub id: String,
  #[serde(rename(deserialize = "Name"))]
  pub name: String,
}

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
pub mod rocket;
