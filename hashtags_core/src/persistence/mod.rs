use super::error::Error;
use super::model;
use std::result::Result;
use std::vec::Vec;

pub mod sqlite;

pub trait Persistence {
    fn create_note(&mut self, _: &str, _: Vec<&str>) -> Result<(), Error>;
    fn query_notes(&self, _: &Vec<&str>, _: &Vec<&str>) -> Result<Vec<model::Note>, Error>;
    fn update_note_by_hash(&mut self, _: &Vec<u8>, _: &str, _: Vec<&str>) -> Result<(), Error>;
}
