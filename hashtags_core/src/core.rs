use super::error::Error;
use super::model::Note;
use super::persistence::sqlite::SqlitePersistence;
use super::persistence::Persistence;
use super::tag::extract_tags;
use super::tokenizer::simple::SimpleTokenizer;
use super::tokenizer::Tokenizer;
use std::boxed::Box;
use std::vec::Vec;

pub struct HashTags {
    p: Box<dyn Persistence>,
}

impl HashTags {
    pub fn new(db_path: &str) -> Result<HashTags, Error> {
        let p = Box::new(SqlitePersistence::new(db_path)?);
        Ok(HashTags { p })
    }

    pub fn create(&mut self, note: &str) -> Result<(), Error> {
        let tags = extract_tags(note)?;
        self.p.create_note(note, tags)
    }

    pub fn query(&self, method: &str, filter: &str) -> Result<Vec<Note>, Error> {
        let t = match method {
            "simple" => SimpleTokenizer::new(),
            ut => return Err(Error::GenericError(format!("unknown tokenizer: {}", ut))),
        };
        let f = t.tokenize(filter)?;
        Ok(self.p.query_notes(&f.ands, &f.ors)?)
    }

    pub fn update(&mut self, note: &str, hash: Vec<u8>) -> Result<(), Error> {
        let tags = extract_tags(note)?;
        self.p.update_note_by_hash(&hash, note, tags)
    }
}
