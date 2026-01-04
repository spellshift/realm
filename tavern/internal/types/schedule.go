package types

// Schedule defines the schedule configuration for a task.
type Schedule struct {
	NewHost   bool   `json:"new_host"`
	NewBeacon bool   `json:"new_beacon"`
	Cron      string `json:"cron"`
}
