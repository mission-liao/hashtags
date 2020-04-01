pub mod simple;

use std::result::Result;
use super::error::Error;

pub struct Filter<'a> {
    pub ands: Vec<&'a str>,
    pub ors: Vec<&'a str>,
}

pub trait Tokenizer {
    fn tokenize<'a>(&self, _: &'a str) -> Result<Filter<'a>, Error>;
}
