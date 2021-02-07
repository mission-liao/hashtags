package persist

import (
	"database/sql"
	"io"
	"time"
)

type Note struct {
	Hash    []byte       `db:"hash"`
	Content string       `db:"content"`
	Created time.Time    `db:"time_created"`
	Updated sql.NullTime `db:"time_updated"`
}

type Persistence interface {
	io.Closer

	CreateNote(note string, tags []string) error
	QueryNotes(ands, ors []string) ([]Note, error)
	UpdateNoteByHash(hash []byte, newNote string, newTags []string) error
}
