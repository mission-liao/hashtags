package common

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestExtractTagBasic(t *testing.T) {
	assert.Empty(t, ExtractTags("kdfkjsdkfjsf"))
	assert.Equal(t, []string{"ss", "tt", "yy"}, ExtractTags("ss #ss #tt # sdkjfk #yy"))
	assert.Equal(t, []string{"ss"}, ExtractTags("#ss"))
	assert.Equal(t, []string{"ss", "tt"}, ExtractTags("#ss #tt #ss"))
}

func TestExtractTagUTF8(t *testing.T) {
	assert.Empty(t, ExtractTags("我家門前有小河"))
	assert.Equal(t, []string{"哎呦", "幹嘛", "測試"}, ExtractTags("ss #測試 #哎呦 # sdkjfk #幹嘛"))
	assert.Equal(t, []string{"再測"}, ExtractTags("#再測"))
	assert.Equal(t, []string{"幹嘛", "測試"}, ExtractTags("ss #幹嘛 #測試  # sdkjfk #幹嘛"))
}
