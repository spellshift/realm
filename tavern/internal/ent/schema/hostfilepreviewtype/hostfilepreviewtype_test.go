package hostfilepreviewtype_test

import (
	"bytes"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/ent/schema/hostfilepreviewtype"
)

func TestHostFilePreviewTypeValues(t *testing.T) {
	assert.NotEmpty(t, hostfilepreviewtype.HostFilePreviewType("").Values())
	assert.Contains(t, hostfilepreviewtype.HostFilePreviewType("").Values(), "TEXT")
	assert.Contains(t, hostfilepreviewtype.HostFilePreviewType("").Values(), "IMAGE")
	assert.Contains(t, hostfilepreviewtype.HostFilePreviewType("").Values(), "NONE")
}

func TestHostFilePreviewTypeValue(t *testing.T) {
	val, err := hostfilepreviewtype.Text.Value()
	require.NoError(t, err)
	assert.Equal(t, "TEXT", val)
}

func TestHostFilePreviewTypeMarshalGQL(t *testing.T) {
	var buf bytes.Buffer
	hostfilepreviewtype.Text.MarshalGQL(&buf)
	assert.Equal(t, `"TEXT"`, buf.String())
}

func TestHostFilePreviewTypeUnmarshalGQL(t *testing.T) {
	var pt hostfilepreviewtype.HostFilePreviewType
	assert.NoError(t, pt.UnmarshalGQL("IMAGE"))
	assert.Equal(t, hostfilepreviewtype.Image, pt)
}

func TestHostFilePreviewTypeScan(t *testing.T) {
	tests := []struct {
		name    string
		scanVal any
		want    hostfilepreviewtype.HostFilePreviewType
		wantErr bool
	}{
		{
			name:    "TEXT_String",
			scanVal: "TEXT",
			want:    hostfilepreviewtype.Text,
		},
		{
			name:    "IMAGE_[]uint8",
			scanVal: []uint8("IMAGE"),
			want:    hostfilepreviewtype.Image,
		},
		{
			name:    "NONE_String",
			scanVal: "NONE",
			want:    hostfilepreviewtype.None,
		},
		{
			name:    "Empty",
			scanVal: "",
			want:    hostfilepreviewtype.None,
		},
		{
			name:    "Nil",
			scanVal: nil,
			want:    hostfilepreviewtype.HostFilePreviewType(""),
		},
		{
			name:    "Invalid",
			scanVal: "INVALID",
			wantErr: true,
		},
		{
			name:    "UnknownType",
			scanVal: 42,
			want:    hostfilepreviewtype.None,
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			var pt hostfilepreviewtype.HostFilePreviewType
			err := pt.Scan(tc.scanVal)
			if tc.wantErr {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
				assert.Equal(t, tc.want, pt)
			}
		})
	}
}
