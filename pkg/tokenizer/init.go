package tokenizer

import "fmt"

type Filter struct {
	Ands, Ors []string
}

type Tokenizer interface {
	Tokenize(s string) (Filter, error)
}

func NewTokenizer(tzType string) Tokenizer {
	switch tzType {
	case "simple":
		return &simpleTokenizer{}
	default:
		panic(fmt.Errorf("invalid tzType:%s", tzType))
	}
}
