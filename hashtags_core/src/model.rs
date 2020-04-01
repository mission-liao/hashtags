use std::vec::Vec;
use std::string::String;
use chrono::{DateTime, Utc};
use serde::{Serialize};

#[derive(Serialize)]
pub struct Note {
    pub hash: Vec<u8>,
    pub content: String,
    pub time_created: DateTime<Utc>,
}
