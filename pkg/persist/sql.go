package persist

import (
	"errors"
	"fmt"
	"time"

	"github.com/jmoiron/sqlx"
	"golang.org/x/crypto/sha3"

	_ "github.com/mattn/go-sqlite3"
)

type sqlitePersist struct {
	db *sqlx.DB
}

func NewSqlitePersist(path string) (p Persistence, err error) {
	db, err := sqlx.Connect("sqlite3", path)
	if err != nil {
		return
	}
	sp := &sqlitePersist{db: db}
	if err = sp.init(); err != nil {
		return
	}
	p = sp
	return
}

func (p *sqlitePersist) CreateNote(note string, tags []string) (err error) {
	h := sha3.New256()
	h.Write([]byte(note))
	var (
		noteHash = h.Sum(nil)
		tx       = p.db.MustBegin()
	)
	defer func() {
		if err != nil {
			if rollbackErr := tx.Rollback(); rollbackErr != nil {
				panic(fmt.Errorf("rollback failed: %v %v", rollbackErr, err))
			}
			return
		}
		err = tx.Commit()
	}()
	if _, err = tx.Exec(
		"INSERT INTO notes (hash, content, time_created) VALUES(?1, ?2, ?3)",
		noteHash, note, time.Now().UTC(),
	); err != nil {
		return
	}
	if err = p.insertTags(tx, tags, noteHash); err != nil {
		return err
	}
	return
}

func (p *sqlitePersist) Close() error {
	err := p.db.Close()
	p.db = nil
	return err
}

func (p *sqlitePersist) QueryNotes(ands, ors []string) ([]Note, error) {
	q, err := p.prepareNotesQueryStmt(ands, ors)
	if err != nil {
		return nil, err
	}
	var (
		notes []Note
		args  []interface{}
	)
	for _, a := range ands {
		args = append(args, a)
	}
	for _, o := range ors {
		args = append(args, o)
	}
	p.db.Select(&notes, q, args...)
	return notes, nil
}

func (p *sqlitePersist) UpdateNoteByHash(hash []byte, newNote string, newTags []string) (err error) {
	h := sha3.New256()
	h.Write([]byte(newNote))
	var (
		newNoteHash = h.Sum(nil)
		tx          = p.db.MustBegin()
	)
	defer func() {
		if err != nil {
			if rollbackErr := tx.Rollback(); rollbackErr != nil {
				panic(fmt.Errorf("rollback failed: %v %v", rollbackErr, err))
			}
			return
		}
		err = tx.Commit()
	}()
	if _, err = tx.Exec(
		"UPDATE notes SET hash=?, content=?, time_updated=? WHERE hash=?",
		newNoteHash, newNote, time.Now().UTC(), hash,
	); err != nil {
		return
	}
	if _, err = tx.Exec("DELETE FROM relations WHERE note_hash=?", hash); err != nil {
		return
	}
	if err = p.insertTags(tx, newTags, newNoteHash); err != nil {
		return
	}
	return
}

func (p *sqlitePersist) init() error {
	if err := p.db.Ping(); err != nil {
		return err
	}
	if _, err := p.db.Exec(`
		CREATE TABLE IF NOT EXISTS notes (
			hash                 BLOB PRIMARY KEY,
            content              TEXT NOT NULL,
            time_created         DATETIME NOT NULL,
            time_updated         DATETIME
		)`,
	); err != nil {
		return err
	}
	if _, err := p.db.Exec(`
		CREATE TABLE IF NOT EXISTS tags (
			name TEXT PRIMARY KEY
		)`,
	); err != nil {
		return err
	}
	if _, err := p.db.Exec(`
		CREATE TABLE IF NOT EXISTS relations (
			tag_name               TEXT,
            note_hash              BLOB,
            FOREIGN KEY(tag_name)  REFERENCES tags(name), 
            FOREIGN KEY(note_hash) REFERENCES notes(hash),
            PRIMARY KEY(tag_name, note_hash)
		)`,
	); err != nil {
		return err
	}
	return nil
}

func (p *sqlitePersist) prepareNotesQueryStmt(ands, ors []string) (string, error) {
	if len(ands) == 0 && len(ors) == 0 {
		return "", errors.New("no filter provided")
	}
	var (
		stmt         = "select * from notes where hash in ("
		addIntersect bool
		addUnion     bool
	)
	for range ands {
		if addIntersect {
			stmt += " INTERSECT "
		}
		stmt += "SELECT note_hash FROM relations WHERE tag_name=?"
		addIntersect = true
	}
	for range ors {
		if addUnion {
			stmt += " UNION "
		} else if len(ands) > 0 {
			stmt += " INTERSECT SELECT * FROM ("
		}
		stmt += "SELECT note_hash FROM relations WHERE tag_name=?"
		addUnion = true
	}
	if len(ands) > 0 && len(ors) > 0 {
		stmt += ")"
	}
	stmt += ") ORDER BY time_created DESC"
	return stmt, nil
}

func (p *sqlitePersist) insertTags(tx *sqlx.Tx, tags []string, hash []byte) error {
	for _, t := range tags {
		if _, err := tx.Exec(`
			INSERT INTO tags (name)
				SELECT ?
					WHERE NOT EXISTS (
						SELECT 1 FROM tags WHERE name = ?
				)`, t, t,
		); err != nil {
			return err
		}
		if _, err := tx.Exec(
			"INSERT INTO relations (tag_name, note_hash) VALUES(?, ?)",
			t, hash,
		); err != nil {
			return err
		}
	}
	return nil
}
