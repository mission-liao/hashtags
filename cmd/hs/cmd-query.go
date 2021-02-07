package main

import (
	"encoding/base64"
	"encoding/json"
	"fmt"
	"time"

	"github.com/mission-liao/hashtags/pkg/persist"
	"github.com/mission-liao/hashtags/pkg/tokenizer"
	"github.com/spf13/cobra"
)

var (
	queryCmd = &cobra.Command{
		Use:          "query",
		Short:        "query notes",
		RunE:         runQueryCmd,
		SilenceUsage: true,
	}

	queryArgs = struct {
		method string
		filter string
		output string
	}{}
)

func runQueryCmd(cmd *cobra.Command, args []string) (err error) {
	p, err := persist.NewSqlitePersist(getDBPath())
	if err != nil {
		return
	}
	defer func() {
		if errClose := p.Close(); errClose != nil {
			if err == nil {
				err = errClose
			} else {
				panic(fmt.Errorf("failed to close DB: %v %v", errClose, err))
			}
		}
	}()
	f, err := tokenizer.NewTokenizer("simple").Tokenize(queryArgs.filter)
	if err != nil {
		return
	}
	notes, err := p.QueryNotes(f.Ands, f.Ors)
	if err != nil {
		return
	}
	switch queryArgs.output {
	case "json":
		var b []byte
		b, err = json.Marshal(notes)
		if err != nil {
			return
		}
		fmt.Println(string(b))
	case "simple":
		for _, n := range notes {
			fmt.Println(n.Content)
			fmt.Println(SEP_SIMPLE)
			fmt.Printf("%v, Hash: %v\n", n.Created.UTC().Truncate(1*time.Second), base64.StdEncoding.EncodeToString(n.Hash))
			fmt.Println(SEP_EQUAL)
		}
	case "concise":
		for _, n := range notes {
			fmt.Println(n.Content)
		}
	default:
		err = fmt.Errorf("unknown output format: %v", queryArgs.output)
		return
	}
	return
}

func init() {
	queryCmd.Flags().StringVarP(
		&queryArgs.method,
		"method", "m",
		"",
		"query method: ['simple']",
	)
	queryCmd.Flags().StringVarP(
		&queryArgs.filter,
		"filter_string", "f",
		"",
		"filter string, ex. 'a,b,c|d|e' -> 'a and b and c and (d or e)",
	)
	queryCmd.Flags().StringVarP(
		&queryArgs.output,
		"output_format", "o",
		"",
		"output format: ['simple', 'json', 'concise']",
	)
}
