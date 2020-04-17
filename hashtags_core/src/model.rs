use chrono::{DateTime, Utc};
use serde::Serialize;
use std::string::String;
use std::vec::Vec;

#[derive(Serialize)]
pub struct Note {
    pub hash: Vec<u8>,
    pub content: String,
    pub time_created: DateTime<Utc>,
    pub time_updated: Option<DateTime<Utc>>,
}
