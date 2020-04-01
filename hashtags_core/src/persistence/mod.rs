use std::result::Result;
use std::vec::Vec;
use super::model;
use super::error::Error;

pub mod sqlite;

pub trait Persistence {
    fn create_note(&mut self, _: &str, _: Vec<&str>) -> Result<(), Error>;
    fn query_notes(&self, _: &Vec<&str>, _: &Vec<&str>) -> Result<Vec<model::Note>, Error>;
}
