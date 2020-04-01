use std::result::Result;
use std::vec::Vec;
use std::string::String;
use sha3::{Digest, Sha3_256};
use rusqlite::{params, Connection, ToSql};
use super::super::error::Error;
use super::Persistence;
use super::super::model;
use chrono::prelude::Utc;

pub struct SqlitePersistence {
    conn: Connection,
}

fn prepare_notes_query_stmt(
    and_tags: &Vec<&str>,
    or_tags: &Vec<&str>) -> Result<String, Error> {
    if and_tags.len() == 0 && or_tags.len() == 0 {
        return Err(Error::GenericError("no filter provided".to_string()));
    }
    let mut stmt = String::from("select * from notes where hash in (");
    let mut add_intersect = false;
    for _ in and_tags {
        if add_intersect {
            stmt.push_str(" INTERSECT ");
        }
        stmt.push_str("SELECT note_hash FROM relations WHERE tag_name = ?");
        add_intersect = true;
    }
    let mut add_union = false;
    for _ in or_tags {
        if add_union {
            stmt.push_str(" UNION ");
        } else if and_tags.len() > 0 {
            stmt.push_str(" INTERSECT SELECT * FROM (");
        }
        stmt.push_str("SELECT note_hash FROM relations WHERE tag_name = ?");
        add_union = true;
    }
    if and_tags.len() > 0 && or_tags.len() > 0 {
        stmt.push_str(")");
    }
    stmt.push_str(") ORDER BY time_created DESC");
    Ok(stmt)
}

impl SqlitePersistence {
    pub fn new(path: &str) -> Result<SqlitePersistence, Error> {
        let conn = match Connection::open(path) {
            Ok(conn) => conn,
            Err(e) => return Err(Error::GenericError(e.to_string())),
        };
        match conn.execute(
            "CREATE TABLE IF NOT EXISTS notes (
                hash                 BLOB PRIMARY KEY,
                content              TEXT NOT NULL,
                time_created         DATETIME NOT NULL
             )",
            params![],
        ) {
            Err(e) => return Err(Error::GenericError(e.to_string())),
            _ => (),
        };
        match conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                name TEXT PRIMARY KEY
             )",
            params![],
        ) {
            Err(e) => return Err(Error::GenericError(e.to_string())),
            _ => (),
        };
        match conn.execute(
            "CREATE TABLE IF NOT EXISTS relations (
                tag_name               TEXT,
                note_hash              BLOB,
                FOREIGN KEY(tag_name)  REFERENCES tags(name), 
                FOREIGN KEY(note_hash) REFERENCES notes(hash),
                PRIMARY KEY(tag_name, note_hash)
            )",
            params![],
        ) {
            Err(e) => return Err(Error::GenericError(e.to_string())),
            _ => (),
        };

        Ok(SqlitePersistence{
            conn
        })
    }
}

impl Persistence for SqlitePersistence {
    fn create_note(&mut self, text: &str, tags: Vec<&str>) -> Result<(), Error> {
        let mut hasher = Sha3_256::new();        
        hasher.input(text);
        let hash = hasher.result();
        let tx = match self.conn.transaction() {
            Ok(tx) => tx,
            Err(e) => return Err(Error::GenericError(e.to_string())),
        };
        match tx.execute(
            "INSERT INTO notes (hash, content, time_created) VALUES(?1, ?2, ?3)",
            params![hash.as_ref(), text, Utc::now()],
        ) {
            Err(e) => return Err(Error::GenericError(e.to_string())),
            _ => (),
        };
        for tag in tags {
            match tx.execute(
                "INSERT INTO tags (name)
                    SELECT ?1
                        WHERE NOT EXISTS(
                            SELECT 1 FROM tags WHERE name = ?2)",
                params![tag, tag],
            ) {
                Err(e) => return Err(Error::GenericError(e.to_string())),
                _ => (),
            };
            match tx.execute(
                "INSERT INTO relations (tag_name, note_hash) VALUES(?1, ?2)",
                params![tag, hash.as_ref()],
            ) {
                Err(e) => return Err(Error::GenericError(e.to_string())),
                _ => (),
            };
        }
        match tx.commit() {
            Err(e) => return Err(Error::GenericError(e.to_string())),
            _ => ()
        };
        Ok(())
    }

    fn query_notes(&self,
        and_tags: &Vec<&str>,
        or_tags: &Vec<&str>) -> Result<Vec<model::Note>, Error> {
        let q = prepare_notes_query_stmt(and_tags, or_tags)?;
        let mut params = Vec::<&dyn ToSql>::new();
        for a in and_tags {
            params.push(a);
        }
        for o in or_tags {
            params.push(o);
        }
        let mut stmt = match self.conn.prepare(&q) {
            Ok(s) => s,
            Err(e) => return Err(Error::GenericError(e.to_string())),
        };
        let note_iter = match stmt.query_map(params, |row| {
            Ok(model::Note{
                hash: row.get(0)?,
                content: row.get(1)?,
                time_created: row.get(2)?,
            })
        }) {
            Ok(note_iter) => note_iter,
            Err(e) => return Err(Error::GenericError(e.to_string())),
        };
        let mut notes = Vec::<model::Note>::new();
        for n in note_iter {
            match n {
                Ok(note) => notes.push(note),
                Err(e) => return Err(Error::GenericError(e.to_string())),
            }
        }
        Ok(notes)
    }
}

#[cfg(test)]
mod test {
    use super::SqlitePersistence;
    use super::Persistence;

    #[test]
    fn test_basic() {
        let mut ps = SqlitePersistence::new(":memory:").unwrap();
        assert!(!ps.create_note("content-1", vec!["tag-1", "tag-2", "tag-3", "tag-4"]).is_err());
        // Inserted content should be able to be queried.
        let notes = ps.query_notes(&vec!["tag-1"], &vec![]).unwrap();
        assert!(notes.len() == 1 && notes[0].content == "content-1");
        let notes = ps.query_notes(&vec!["tag-1", "tag-2", "tag-3"], &vec![]).unwrap();
        assert!(notes.len() == 1 && notes[0].content == "content-1");
        let notes = ps.query_notes(&vec!["tag-1", "tag-2", "tag-5"], &vec![]).unwrap();
        assert!(notes.len() == 0); 
        // Duplicate content should be rejected.
        assert!(ps.create_note("content-1", vec![]).is_err());
        // Build more complex scenario.
        assert!(!ps.create_note("content-2", vec!["tag-1", "tag-3", "tag-6"]).is_err());
        assert!(!ps.create_note("content-3", vec!["tag-3", "tag-6"]).is_err());
        // Test AND and OR.
        let notes = ps.query_notes(&vec!["tag-1", "tag-3"], &vec!["tag-4", "tag-6"]).unwrap();
        assert!(notes.len() == 2 && notes[0].content == "content-2" && notes[1].content == "content-1");
        let notes = ps.query_notes(&vec!["tag-1", "tag-3"], &vec!["tag-6"]).unwrap();
        assert!(notes.len() == 1 && notes[0].content == "content-2"); 
    }

    #[test]
    fn test_utf8() {
        let mut ps = SqlitePersistence::new(":memory:").unwrap();
        assert!(!ps.create_note("content-1 #台積電 #2330 #2018 年報", vec!["台積電", "2330", "2018", "現貨"]).is_err());
        assert!(!ps.create_note("content-2 #台達電 #2308 #2018 年報", vec!["台達電", "2308", "2018", "現貨"]).is_err());
        assert!(!ps.create_note("content-3 #0050 #2017", vec!["0050", "2017", "ETF"]).is_err());
        assert!(!ps.create_note("content-4 #台達電 #2308 #2017 年報", vec!["台達電", "2308", "2017", "現貨"]).is_err());
        let notes = ps.query_notes(&vec!["2018", "現貨"], &vec!["台積電", "台達電"]).unwrap();
        assert!(
            notes.len() == 2 &&
            notes[0].content.starts_with("content-2") &&
            notes[1].content.starts_with("content-1"));
        let notes = ps.query_notes(&vec!["2017"], &vec!["現貨", "ETF"]).unwrap();
        assert!(
            notes.len() == 2 &&
            notes[0].content.starts_with("content-4") &&
            notes[1].content.starts_with("content-3"))
    }
}
