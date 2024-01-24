package c2pb_test

import (
	"bytes"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/c2/c2pb"
)

func TestHostPlatformValues(t *testing.T) {
	assert.NotEmpty(t, c2pb.Host_Platform(0).Values())
}

func TestHostPlatformValue(t *testing.T) {
	val, err := c2pb.Host_Platform(0).Value()
	require.NoError(t, err)
	require.NotNil(t, val)
}

func TestHostPlatformMarshalGraphQL(t *testing.T) {
	var buf bytes.Buffer
	c2pb.Host_Platform(0).MarshalGQL(&buf)
	assert.Equal(t, `"PLATFORM_UNSPECIFIED"`, buf.String())
}

func TestHostPlatformUnmarshalGraphQL(t *testing.T) {
	var platform c2pb.Host_Platform
	assert.NoError(t, (*c2pb.Host_Platform).UnmarshalGQL(&platform, `PLATFORM_LINUX`))
	assert.Equal(t, c2pb.Host_PLATFORM_LINUX, platform)
}

func TestHostPlatformScan(t *testing.T) {
	tests := []struct {
		name         string
		scanVal      any
		wantPlatform c2pb.Host_Platform
	}{
		{
			name:         "Linux_String",
			scanVal:      "PLATFORM_LINUX",
			wantPlatform: c2pb.Host_PLATFORM_LINUX,
		},
		{
			name:         "Windows_[]uint8",
			scanVal:      []uint8("PLATFORM_WINDOWS"),
			wantPlatform: c2pb.Host_PLATFORM_WINDOWS,
		},
		{
			name:         "Invalid",
			scanVal:      "NOT_A_PLATFORM",
			wantPlatform: c2pb.Host_PLATFORM_UNSPECIFIED,
		},
		{
			name:         "Empty",
			scanVal:      "",
			wantPlatform: c2pb.Host_PLATFORM_UNSPECIFIED,
		},
		{
			name:         "Nil",
			scanVal:      nil,
			wantPlatform: c2pb.Host_PLATFORM_UNSPECIFIED,
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			platform := c2pb.Host_Platform(0)
			err := (*c2pb.Host_Platform).Scan(&platform, tc.scanVal)
			assert.NoError(t, err)
			assert.Equal(t, tc.wantPlatform, platform)
		})
	}
}
