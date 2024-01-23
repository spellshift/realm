package c2pb_test

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"realm.pub/tavern/internal/c2/c2pb"
)

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
