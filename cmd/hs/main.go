package main

import (
	"os"
	"path"

	"github.com/spf13/cobra"
)

const (
	SEP_SIMPLE = "--------------------------------------------------------------------------------"
	SEP_EQUAL  = "================================================================================"
	PATT_HASH  = ", Hash: "
)

var (
	rootCmd = &cobra.Command{
		Use:          "bomb",
		Short:        "Bomb is a bomb to WebRTC relay servers",
		SilenceUsage: true,
	}
)

func getDBPath() string {
	hd, err := os.UserHomeDir()
	if err != nil {
		panic(err)
	}
	return path.Join(hd, "notes.db")
}

func main() {
	if err := rootCmd.Execute(); err != nil {
		panic(err)
	}
}

func init() {
	rootCmd.AddCommand(createCmd)
	rootCmd.AddCommand(queryCmd)
	rootCmd.AddCommand(updateCmd)
}
