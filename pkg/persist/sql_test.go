package persist

import (
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestSqlitePersistBacis(t *testing.T) {
	s, err := NewSqlitePersist(":memory:")
	assert.NoError(t, err)
	defer func() { assert.NoError(t, s.Close()) }()
	assert.NoError(t, s.CreateNote("content-1", []string{"tag-1", "tag-2", "tag-3", "tag-4"}))
	// Inserted content should be able to be queried.
	notes, err := s.QueryNotes([]string{"tag-1"}, []string{})
	assert.NoError(t, err)
	assert.Len(t, notes, 1)
	assert.Equal(t, "content-1", notes[0].Content)
	notes, err = s.QueryNotes([]string{"tag-1", "tag-2", "tag-3"}, []string{})
	assert.NoError(t, err)
	assert.Len(t, notes, 1)
	assert.Equal(t, "content-1", notes[0].Content)
	notes, err = s.QueryNotes([]string{"tag-1", "tag-2", "tag-5"}, []string{})
	assert.NoError(t, err)
	assert.Len(t, notes, 0)
	// Duplicated content should be rejected.
	assert.Error(t, s.CreateNote("content-1", []string{}))
	// Build more complex scenario.
	assert.NoError(t, s.CreateNote("content-2", []string{"tag-1", "tag-3", "tag-6"}))
	assert.NoError(t, s.CreateNote("content-3", []string{"tag-3", "tag-6"}))
	// Test AND and OR.
	notes, err = s.QueryNotes([]string{"tag-1", "tag-3"}, []string{"tag-4", "tag-6"})
	assert.NoError(t, err)
	assert.Len(t, notes, 2)
	assert.Equal(t, "content-2", notes[0].Content)
	assert.Equal(t, "content-1", notes[1].Content)
	notes, err = s.QueryNotes([]string{"tag-1", "tag-3"}, []string{"tag-6"})
	assert.NoError(t, err)
	assert.Len(t, notes, 1)
	assert.Equal(t, "content-2", notes[0].Content)
}

func TestSqlitePersistUpdateBacis(t *testing.T) {
	s, err := NewSqlitePersist(":memory:")
	assert.NoError(t, err)
	defer func() { assert.NoError(t, s.Close()) }()
	assert.NoError(t, s.CreateNote("content-1", []string{"tag-1", "tag-2", "tag-3", "tag-4"}))
	notes, err := s.QueryNotes([]string{"tag-1"}, []string{})
	assert.NoError(t, err)
	assert.Len(t, notes, 1)
	assert.Equal(t, "content-1", notes[0].Content)
	assert.NoError(t, s.UpdateNoteByHash(notes[0].Hash, "content-2", []string{"tag-1", "tag-2"}))
	notes, err = s.QueryNotes([]string{"tag-1"}, []string{})
	assert.NoError(t, err)
	assert.Len(t, notes, 1)
	assert.Equal(t, "content-2", notes[0].Content)
}

func TestSqlitePersistUTF8(t *testing.T) {
	s, err := NewSqlitePersist(":memory:")
	assert.NoError(t, err)
	defer func() { assert.NoError(t, s.Close()) }()
	assert.NoError(t, s.CreateNote("content-1 #台積電 #2330 #2018 年報", []string{"台積電", "2330", "2018", "現貨"}))
	assert.NoError(t, s.CreateNote("content-2 #台達電 #2308 #2018 年報", []string{"台達電", "2308", "2018", "現貨"}))
	assert.NoError(t, s.CreateNote("content-3 #0050 #2017", []string{"0050", "2017", "ETF"}))
	assert.NoError(t, s.CreateNote("content-4 #台達電 #2308 #2017 年報", []string{"台達電", "2308", "2017", "現貨"}))
	notes, err := s.QueryNotes([]string{"2018", "現貨"}, []string{"台積電", "台達電"})
	assert.NoError(t, err)
	assert.Len(t, notes, 2)
	assert.True(t, strings.HasPrefix(notes[0].Content, "content-2"))
	assert.True(t, strings.HasPrefix(notes[1].Content, "content-1"))
	notes, err = s.QueryNotes([]string{"2017"}, []string{"現貨", "ETF"})
	assert.NoError(t, err)
	assert.Len(t, notes, 2)
	assert.True(t, strings.HasPrefix(notes[0].Content, "content-4"))
	assert.True(t, strings.HasPrefix(notes[1].Content, "content-3"))
}

func TestSqlitePersistUpdateUTF8(t *testing.T) {
	s, err := NewSqlitePersist(":memory:")
	assert.NoError(t, err)
	defer func() { assert.NoError(t, s.Close()) }()
	assert.NoError(t, s.CreateNote("content-1 #台積電 #2330 #2018 年報", []string{"台積電", "2330", "2018", "現貨"}))
	notes, err := s.QueryNotes([]string{"台積電"}, []string{})
	assert.NoError(t, err)
	assert.Len(t, notes, 1)
	assert.True(t, strings.HasPrefix(notes[0].Content, "content-1"))
	assert.NoError(t, s.UpdateNoteByHash(notes[0].Hash, "content-2 #台積電 #2330 #2018 年報", []string{"台積電", "2330", "2018", "現貨"}))
	notes, err = s.QueryNotes([]string{"台積電"}, []string{})
	assert.NoError(t, err)
	assert.Len(t, notes, 1)
	assert.True(t, strings.HasPrefix(notes[0].Content, "content-2"))
}
