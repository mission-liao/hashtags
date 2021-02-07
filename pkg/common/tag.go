package common

import (
	"regexp"
	"sort"
	"strings"
)

func ExtractTags(note string) []string {
	m := regexp.MustCompile(`((^|\s)#[^\s\t\.\?#,]+)`)
	tags := m.FindAllString(note, -1)
	tagSet := map[string]struct{}{}
	for _, t := range tags {
		t = strings.Trim(t, " ")[1:]
		tagSet[t] = struct{}{}
	}
	ret := []string{}
	for t := range tagSet {
		ret = append(ret, t)
	}
	sort.Strings(ret)
	return ret
}
