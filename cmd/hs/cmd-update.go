package main

import (
	"encoding/base64"
	"errors"
	"fmt"
	"strings"

	"github.com/mission-liao/hashtags/pkg/common"
	"github.com/mission-liao/hashtags/pkg/persist"
	"github.com/spf13/cobra"
)

var (
	updateCmd = &cobra.Command{
		Use:          "update",
		Short:        "update one note",
		RunE:         runUpdateCmd,
		SilenceUsage: true,
	}

	updateArgs = struct {
		note string
	}{}
)

func runUpdateCmd(cmd *cobra.Command, args []string) (err error) {
	i := strings.Index(updateArgs.note, SEP_SIMPLE)
	if i == -1 {
		err = errors.New("unable to locate simple_seq")
		return
	}
	realNote := updateArgs.note[:i-1] // EOL
	meta := updateArgs.note[i+len(SEP_SIMPLE):]
	i = strings.Index(meta, PATT_HASH)
	if i == -1 {
		err = errors.New("unable to locate hash in meta")
		return
	}
	tags := common.ExtractTags(realNote)
	if len(tags) == 0 {
		err = errors.New("empty tags extracted")
		return
	}
	hash, err := base64.StdEncoding.DecodeString(meta[i+len(PATT_HASH) : len(meta)-len(SEP_EQUAL)-1]) // EOL
	if err != nil {
		return
	}
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
	if err = p.UpdateNoteByHash(hash, realNote, tags); err != nil {
		return
	}
	return
}

func init() {
	updateCmd.Flags().StringVarP(
		&updateArgs.note,
		"note", "n",
		"",
		"note to update, should appends with note's meta",
	)
}
