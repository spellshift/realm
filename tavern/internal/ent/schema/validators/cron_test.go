package validators_test

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"realm.pub/tavern/internal/ent/schema/validators"
)

func TestNewCronSchedule(t *testing.T) {
	tests := []struct {
		name    string
		expr    string
		wantErr bool
	}{
		{name: "Empty", expr: "", wantErr: false},
		{name: "EveryMinute", expr: "* * * * *", wantErr: false},
		{name: "Hourly", expr: "0 * * * *", wantErr: false},
		{name: "SteppedRange", expr: "0-30/5 9-17/2 * * MON-FRI", wantErr: false},
		{name: "SundayZero", expr: "0 0 * * 0", wantErr: false},
		{name: "SundaySevenRejected", expr: "0 0 * * 7", wantErr: true},
		{name: "OutOfBoundsMinute", expr: "99 * * * *", wantErr: true},
		{name: "WrongFieldCount", expr: "* * *", wantErr: true},
		{name: "Garbage", expr: "not a cron", wantErr: true},
	}

	validate := validators.NewCronSchedule()
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			err := validate(tc.expr)
			if tc.wantErr {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
			}
		})
	}
}
