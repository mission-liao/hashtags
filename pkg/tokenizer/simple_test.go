package tokenizer

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestSimpleTokenizerBasic(t *testing.T) {
	st := NewTokenizer("simple")
	f, err := st.Tokenize("a,b,c,d")
	assert.NoError(t, err)
	assert.Equal(t, []string{"a", "b", "c", "d"}, f.Ands)
	assert.Empty(t, f.Ors)
	f, err = st.Tokenize("a|b|c")
	assert.NoError(t, err)
	assert.Empty(t, f.Ands)
	assert.Equal(t, []string{"a", "b", "c"}, f.Ors)
	f, err = st.Tokenize("a,b,c|d|e")
	assert.NoError(t, err)
	assert.Equal(t, []string{"a", "b"}, f.Ands)
	assert.Equal(t, []string{"c", "d", "e"}, f.Ors)
}

func TestSimpleTokenizerUTF8(t *testing.T) {
	st := NewTokenizer("simple")
	f, err := st.Tokenize("台積電,聯電,ETF")
	assert.NoError(t, err)
	assert.Equal(t, []string{"台積電", "聯電", "ETF"}, f.Ands)
	assert.Empty(t, f.Ors)
	f, err = st.Tokenize("台積電,聯電,ETF|台達電|華碩")
	assert.NoError(t, err)
	assert.Equal(t, []string{"台積電", "聯電"}, f.Ands)
	assert.Equal(t, []string{"ETF", "台達電", "華碩"}, f.Ors)
}
