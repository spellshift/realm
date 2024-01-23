package c2pb_test

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"realm.pub/tavern/internal/c2/c2pb"
)

func TestProcessStatusScan(t *testing.T) {
	tests := []struct {
		name       string
		scanVal    any
		wantStatus c2pb.Process_Status
	}{
		{
			name:       "RUN_String",
			scanVal:    "STATUS_RUN",
			wantStatus: c2pb.Process_STATUS_RUN,
		},
		{
			name:       "IDLE_[]uint8",
			scanVal:    []uint8("STATUS_IDLE"),
			wantStatus: c2pb.Process_STATUS_IDLE,
		},
		{
			name:       "Invalid",
			scanVal:    "NOT_A_STATUS",
			wantStatus: c2pb.Process_STATUS_UNKNOWN,
		},
		{
			name:       "Empty",
			scanVal:    "",
			wantStatus: c2pb.Process_STATUS_UNSPECIFIED,
		},
		{
			name:       "Nil",
			scanVal:    nil,
			wantStatus: c2pb.Process_STATUS_UNSPECIFIED,
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			status := c2pb.Process_Status(0)
			err := (*c2pb.Process_Status).Scan(&status, tc.scanVal)
			assert.NoError(t, err)
			assert.Equal(t, tc.wantStatus, status)
		})
	}
}
