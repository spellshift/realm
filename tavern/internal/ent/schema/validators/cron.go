package validators

import (
	"fmt"

	"github.com/robfig/cron/v3"
)

var cronScheduleParser = cron.NewParser(cron.Minute | cron.Hour | cron.Dom | cron.Month | cron.Dow)

// NewCronSchedule returns a validator that allows an empty string or a valid
// 5-field cron expression accepted by robfig/cron/v3 (matching tome automation).
func NewCronSchedule() func(string) error {
	return func(expr string) error {
		if expr == "" {
			return nil
		}
		if _, err := cronScheduleParser.Parse(expr); err != nil {
			return fmt.Errorf("invalid cron schedule %q: %w", expr, err)
		}
		return nil
	}
}
