package tokenizer

import (
	"errors"
	"strings"
)

type simpleTokenizer struct {
}

func (t *simpleTokenizer) Tokenize(s string) (f Filter, err error) {
	if len(s) == 0 {
		err = errors.New("empty filter-string")
		return
	}
	var (
		ands     = strings.Split(s, ",")
		andsLast = ands[len(ands)-1]
		ors      []string
	)
	if strings.Contains(andsLast, "|") {
		ands = ands[:len(ands)-1]
		ors = strings.Split(andsLast, "|")
	}
	f = Filter{Ands: ands, Ors: ors}
	return
}
