use super::super::error::Error;
use super::super::model;
use super::Persistence;
use chrono::prelude::Utc;
use rusqlite::Result as RusqResult;
use rusqlite::{params, Connection, ToSql, Transaction};
use sha3::{Digest, Sha3_256};
use std::result::Result;
use std::string::String;
use std::vec::Vec;

pub struct SqlitePersistence {
    conn: Connection,
}

fn prepare_notes_query_stmt(and_tags: &Vec<&str>, or_tags: &Vec<&str>) -> Result<String, Error> {
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

fn insert_tags(tx: &Transaction, tags: &Vec<&str>, hash: &Vec<u8>) -> RusqResult<()> {
    for tag in tags {
        tx.execute(
            "INSERT INTO tags (name)
                SELECT ?1
                    WHERE NOT EXISTS(
                        SELECT 1 FROM tags WHERE name = ?2)",
            params![tag, tag],
        )?;
        tx.execute(
            "INSERT INTO relations (tag_name, note_hash) VALUES(?1, ?2)",
            params![tag, hash],
        )?;
    }
    Ok(())
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
                time_created         DATETIME NOT NULL,
                time_updated         DATETIME
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

        Ok(SqlitePersistence { conn })
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
        match insert_tags(&tx, &tags, &hash.to_vec()) {
            Err(e) => return Err(Error::GenericError(e.to_string())),
            _ => (),
        }
        match tx.commit() {
            Err(e) => return Err(Error::GenericError(e.to_string())),
            _ => (),
        };
        Ok(())
    }

    fn query_notes(
        &self,
        and_tags: &Vec<&str>,
        or_tags: &Vec<&str>,
    ) -> Result<Vec<model::Note>, Error> {
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
            Ok(model::Note {
                hash: row.get(0)?,
                content: row.get(1)?,
                time_created: row.get(2)?,
                time_updated: None,
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

    fn update_note_by_hash(
        &mut self,
        hash: &Vec<u8>,
        text: &str,
        tags: Vec<&str>,
    ) -> Result<(), Error> {
        let tx = match self.conn.transaction() {
            Ok(tx) => tx,
            Err(e) => return Err(Error::GenericError(e.to_string())),
        };
        // Make sure the note corresponding to that hash exists.
        let mut hasher = Sha3_256::new();
        hasher.input(text);
        let new_hash = hasher.result();
        match tx.execute(
            "UPDATE
                notes
             SET
                hash=?1,
                content=?2,
                time_updated=?3
            WHERE
                hash=?4",
            params![new_hash.as_ref(), text, Utc::now(), hash],
        ) {
            Ok(updated) => {
                if updated == 0 {
                    return Err(Error::GenericError(
                        "unable to locate row by hash".to_string(),
                    ));
                }
            }
            Err(e) => return Err(Error::GenericError(e.to_string())),
        };
        // Delete all rows in tags
        match tx.execute("DELETE FROM relations WHERE note_hash = ?1", params![hash]) {
            Err(e) => return Err(Error::GenericError(e.to_string())),
            _ => (),
        };
        match insert_tags(&tx, &tags, &new_hash.to_vec()) {
            Err(e) => return Err(Error::GenericError(e.to_string())),
            _ => (),
        };
        match tx.commit() {
            Err(e) => return Err(Error::GenericError(e.to_string())),
            _ => (),
        };
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::Persistence;
    use super::SqlitePersistence;

    #[test]
    fn test_basic() {
        let mut ps = SqlitePersistence::new(":memory:").unwrap();
        assert!(!ps
            .create_note("content-1", vec!["tag-1", "tag-2", "tag-3", "tag-4"])
            .is_err());
        // Inserted content should be able to be queried.
        let notes = ps.query_notes(&vec!["tag-1"], &vec![]).unwrap();
        assert!(notes.len() == 1 && notes[0].content == "content-1");
        let notes = ps
            .query_notes(&vec!["tag-1", "tag-2", "tag-3"], &vec![])
            .unwrap();
        assert!(notes.len() == 1 && notes[0].content == "content-1");
        let notes = ps
            .query_notes(&vec!["tag-1", "tag-2", "tag-5"], &vec![])
            .unwrap();
        assert!(notes.len() == 0);
        // Duplicate content should be rejected.
        assert!(ps.create_note("content-1", vec![]).is_err());
        // Build more complex scenario.
        assert!(!ps
            .create_note("content-2", vec!["tag-1", "tag-3", "tag-6"])
            .is_err());
        assert!(!ps.create_note("content-3", vec!["tag-3", "tag-6"]).is_err());
        // Test AND and OR.
        let notes = ps
            .query_notes(&vec!["tag-1", "tag-3"], &vec!["tag-4", "tag-6"])
            .unwrap();
        assert!(
            notes.len() == 2 && notes[0].content == "content-2" && notes[1].content == "content-1"
        );
        let notes = ps
            .query_notes(&vec!["tag-1", "tag-3"], &vec!["tag-6"])
            .unwrap();
        assert!(notes.len() == 1 && notes[0].content == "content-2");
    }

    #[test]
    fn test_update_basic() {
        let mut ps = SqlitePersistence::new(":memory:").unwrap();
        assert!(!ps
            .create_note("content-1", vec!["tag-1", "tag-2", "tag-3", "tag-4"])
            .is_err());
        let notes = ps.query_notes(&vec!["tag-1"], &vec![]).unwrap();
        assert!(notes.len() == 1 && notes[0].content == "content-1");
        ps.update_note_by_hash(&notes[0].hash, "content-2", vec!["tag-1", "tag-2"])
            .unwrap();
        let notes = ps.query_notes(&vec!["tag-1"], &vec![]).unwrap();
        assert!(notes.len() == 1 && notes[0].content == "content-2")
    }

    #[test]
    fn test_utf8() {
        let mut ps = SqlitePersistence::new(":memory:").unwrap();
        assert!(!ps
            .create_note(
                "content-1 #台積電 #2330 #2018 年報",
                vec!["台積電", "2330", "2018", "現貨"]
            )
            .is_err());
        assert!(!ps
            .create_note(
                "content-2 #台達電 #2308 #2018 年報",
                vec!["台達電", "2308", "2018", "現貨"]
            )
            .is_err());
        assert!(!ps
            .create_note("content-3 #0050 #2017", vec!["0050", "2017", "ETF"])
            .is_err());
        assert!(!ps
            .create_note(
                "content-4 #台達電 #2308 #2017 年報",
                vec!["台達電", "2308", "2017", "現貨"]
            )
            .is_err());
        let notes = ps
            .query_notes(&vec!["2018", "現貨"], &vec!["台積電", "台達電"])
            .unwrap();
        assert!(
            notes.len() == 2
                && notes[0].content.starts_with("content-2")
                && notes[1].content.starts_with("content-1")
        );
        let notes = ps.query_notes(&vec!["2017"], &vec!["現貨", "ETF"]).unwrap();
        assert!(
            notes.len() == 2
                && notes[0].content.starts_with("content-4")
                && notes[1].content.starts_with("content-3")
        )
    }

    #[test]
    fn test_update_utf8() {
        let mut ps = SqlitePersistence::new(":memory:").unwrap();
        assert!(!ps
            .create_note(
                "content-1 #台積電 #2330 #2018 年報",
                vec!["台積電", "2330", "2018", "現貨"]
            )
            .is_err());
        let notes = ps.query_notes(&vec!["台積電"], &vec![]).unwrap();
        assert!(notes.len() == 1 && notes[0].content.starts_with("content-1"));
        ps.update_note_by_hash(
            &notes[0].hash,
            "content-2 #台積電 #2330 #2018 年報",
            vec!["台積電", "2330", "2018", "現貨"],
        )
        .unwrap();
        let notes = ps.query_notes(&vec!["台積電"], &vec![]).unwrap();
        assert!(notes.len() == 1 && notes[0].content.starts_with("content-2"));
    }
}
