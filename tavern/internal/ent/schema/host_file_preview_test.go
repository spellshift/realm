package schema

import (
	"encoding/base64"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"realm.pub/tavern/internal/ent/schema/hostfilepreviewtype"
)

func TestIsImage(t *testing.T) {
	tests := []struct {
		name    string
		content []byte
		want    bool
	}{
		{
			name:    "PNG",
			content: append([]byte{0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A}, []byte("rest of png")...),
			want:    true,
		},
		{
			name:    "JPEG",
			content: append([]byte{0xFF, 0xD8, 0xFF, 0xE0}, []byte("rest of jpeg")...),
			want:    true,
		},
		{
			name:    "GIF87a",
			content: []byte("GIF87a rest of gif"),
			want:    true,
		},
		{
			name:    "GIF89a",
			content: []byte("GIF89a rest of gif"),
			want:    true,
		},
		{
			name:    "BMP",
			content: append([]byte{0x42, 0x4D}, []byte("rest of bmp")...),
			want:    true,
		},
		{
			name:    "WEBP",
			content: append([]byte("RIFF\x00\x00\x00\x00WEBP"), []byte("rest of webp")...),
			want:    true,
		},
		{
			name:    "PlainText",
			content: []byte("Hello, world!"),
			want:    false,
		},
		{
			name:    "Empty",
			content: []byte{},
			want:    false,
		},
		{
			name:    "TooShort",
			content: []byte{0xFF, 0xD8},
			want:    false,
		},
		{
			name:    "RandomBinary",
			content: []byte{0x01, 0x02, 0x03, 0x04},
			want:    false,
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			assert.Equal(t, tc.want, isImage(tc.content))
		})
	}
}

func TestIsHumanReadableText(t *testing.T) {
	tests := []struct {
		name    string
		content []byte
		want    bool
	}{
		{
			name:    "PlainASCII",
			content: []byte("Hello, world!\n"),
			want:    true,
		},
		{
			name:    "TabsAndNewlines",
			content: []byte("col1\tcol2\nval1\tval2\n"),
			want:    true,
		},
		{
			name:    "CarriageReturnLF",
			content: []byte("Hello\r\nWorld"),
			want:    true,
		},
		{
			name:    "UTF8Text",
			content: []byte("\xe4\xb8\x96\xe7\x95\x8c"),
			want:    true,
		},
		{
			name:    "NullByte",
			content: []byte("Hello\x00World"),
			want:    false,
		},
		{
			name:    "BinaryControlChars",
			content: []byte{0x01, 0x02, 0x03},
			want:    false,
		},
		{
			name:    "Empty",
			content: []byte{},
			want:    false,
		},
		{
			name:    "EscapeChar",
			content: []byte("Hello\x1bWorld"),
			want:    false,
		},
		{
			name:    "OnlyPrintable",
			content: []byte("abcdefghijklmnopqrst"),
			want:    true,
		},
		{
			name:    "TextLongerThan20Bytes_BinaryAfter20",
			content: append([]byte("abcdefghijklmnopqrstu"), []byte{0x00, 0x01, 0x02}...),
			want:    true, // Only first 20 bytes checked
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			assert.Equal(t, tc.want, isHumanReadableText(tc.content))
		})
	}
}

func TestDerivePreview(t *testing.T) {
	tests := []struct {
		name            string
		content         []byte
		wantType        hostfilepreviewtype.HostFilePreviewType
		wantPreviewLen  int
		wantPreviewEmpty bool
	}{
		{
			name:             "Empty",
			content:          nil,
			wantType:         hostfilepreviewtype.None,
			wantPreviewEmpty: true,
		},
		{
			name:    "SmallText",
			content: []byte("Hello, world!"),
			wantType: hostfilepreviewtype.Text,
			wantPreviewLen: 13,
		},
		{
			name:    "LargeText",
			content: []byte(strings.Repeat("a", maxPreviewSize+100)),
			wantType: hostfilepreviewtype.Text,
			wantPreviewLen: maxPreviewSize,
		},
		{
			name:    "TextExactly512KB",
			content: []byte(strings.Repeat("x", maxPreviewSize)),
			wantType: hostfilepreviewtype.Text,
			wantPreviewLen: maxPreviewSize,
		},
		{
			name:    "SmallPNG",
			content: append([]byte{0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A}, []byte("small png data")...),
			wantType: hostfilepreviewtype.Image,
			wantPreviewLen: base64.StdEncoding.EncodedLen(8 + len("small png data")),
		},
		{
			name:    "SmallJPEG",
			content: append([]byte{0xFF, 0xD8, 0xFF, 0xE0}, []byte("jpeg data")...),
			wantType: hostfilepreviewtype.Image,
			wantPreviewLen: base64.StdEncoding.EncodedLen(4 + len("jpeg data")),
		},
		{
			name:             "LargePNG",
			content:          append([]byte{0x89, 0x50, 0x4E, 0x47}, make([]byte, maxPreviewSize+1)...),
			wantType:         hostfilepreviewtype.None,
			wantPreviewEmpty: true,
		},
		{
			name:    "ImageExactly512KB",
			content: func() []byte {
				b := make([]byte, maxPreviewSize)
				copy(b, []byte{0x89, 0x50, 0x4E, 0x47})
				return b
			}(),
			wantType:   hostfilepreviewtype.Image,
			wantPreviewLen: base64.StdEncoding.EncodedLen(maxPreviewSize),
		},
		{
			name:             "BinaryContent",
			content:          []byte{0x00, 0x01, 0x02, 0x03, 0x04},
			wantType:         hostfilepreviewtype.None,
			wantPreviewEmpty: true,
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			gotType, gotPreview := derivePreview(tc.content)
			assert.Equal(t, tc.wantType, gotType)
			if tc.wantPreviewEmpty {
				assert.Empty(t, gotPreview)
			} else {
				assert.Len(t, gotPreview, tc.wantPreviewLen)
			}
		})
	}
}

func TestDerivePreview_TextContent(t *testing.T) {
	content := []byte("Hello, world!")
	gotType, gotPreview := derivePreview(content)
	assert.Equal(t, hostfilepreviewtype.Text, gotType)
	assert.Equal(t, "Hello, world!", gotPreview)
}

func TestDerivePreview_ImageBase64(t *testing.T) {
	content := append([]byte{0xFF, 0xD8, 0xFF, 0xE0}, []byte("jpeg data")...)
	gotType, gotPreview := derivePreview(content)
	assert.Equal(t, hostfilepreviewtype.Image, gotType)
	assert.Equal(t, base64.StdEncoding.EncodeToString(content), gotPreview)
}
