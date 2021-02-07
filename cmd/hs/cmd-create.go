package main

import (
	"errors"
	"fmt"

	"github.com/mission-liao/hashtags/pkg/common"
	"github.com/mission-liao/hashtags/pkg/persist"
	"github.com/spf13/cobra"
)

var (
	createCmd = &cobra.Command{
		Use:          "create",
		Short:        "create a new note",
		RunE:         runCreateCmd,
		SilenceUsage: true,
	}

	createArgs = struct {
		note string
	}{}
)

func runCreateCmd(cmd *cobra.Command, args []string) (err error) {
	tags := common.ExtractTags(createArgs.note)
	if len(tags) == 0 {
		err = errors.New("empty tags extracted")
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
	if err = p.CreateNote(createArgs.note, tags); err != nil {
		return
	}
	return
}

func init() {
	createCmd.Flags().StringVarP(
		&createArgs.note,
		"note", "n",
		"",
		"note to create",
	)
}
