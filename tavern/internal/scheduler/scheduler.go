package scheduler

import (
	"context"
	"errors"
	"fmt"
	"maps"
	"net/url"
	"slices"
	"sync"
	"time"
)

// ErrJobExists is returned when attempting to create a job that already exists.
var ErrJobExists = errors.New("job already exists")

// HTTPTarget defines an HTTP endpoint to invoke when a scheduled job fires.
type HTTPTarget struct {
	URL     string
	Method  string
	Headers map[string]string
	Body    []byte
}

// Job represents a unit of work to be run on a schedule.
type Job struct {
	// Name uniquely identifies the job within a scheduler.
	Name string

	// Schedule is a cron expression (e.g. "*/5 * * * *").
	Schedule string

	// HTTPTarget is the HTTP endpoint to call when the job fires.
	HTTPTarget HTTPTarget
}

// OnceJob represents a unit of work to be run once at a specific time.
type OnceJob struct {
	// Name uniquely identifies the job within a scheduler.
	Name string

	// At is the time at which the job should fire.
	// If At is in the past, the job fires immediately.
	At time.Time

	// HTTPTarget is the HTTP endpoint to call when the job fires.
	HTTPTarget HTTPTarget
}

// Scheduler manages scheduled jobs.
type Scheduler interface {
	// Schedule creates a job that operates on a schedule.
	// If a job with the same name already exists, it returns an error.
	Schedule(ctx context.Context, job Job) error

	// ScheduleAt creates a one-time job that fires at a specific time.
	// If a job with the same name already exists, it returns an error.
	ScheduleAt(ctx context.Context, job OnceJob) error

	// Close releases any resources held by the scheduler.
	Close() error
}

// Driver is the interface that must be implemented by a scheduler driver.
// It follows the database/sql pattern for driver registration.
type Driver interface {
	// Open returns a new Scheduler using the provided URI.
	Open(ctx context.Context, uri *url.URL) (Scheduler, error)
}

var (
	mu      sync.RWMutex
	drivers = make(map[string]Driver)
)

// Register makes a Driver available by the provided scheme name.
// If Register is called twice with the same name or if driver is nil, it panics.
func Register(name string, driver Driver) {
	mu.Lock()
	defer mu.Unlock()
	if driver == nil {
		panic("scheduler: Register driver is nil")
	}
	if _, dup := drivers[name]; dup {
		panic("scheduler: Register called twice for driver " + name)
	}
	drivers[name] = driver
}

// Drivers returns a sorted list of the names of the registered drivers.
func Drivers() []string {
	mu.RLock()
	defer mu.RUnlock()
	return slices.Sorted(maps.Keys(drivers))
}

// New opens a Scheduler identified by its URI.
// The URI scheme selects the driver (e.g. "gcp://..." or "mem://...").
func New(ctx context.Context, uri string) (Scheduler, error) {
	parsed, err := url.Parse(uri)
	if err != nil {
		return nil, fmt.Errorf("scheduler: invalid uri %q: %w", uri, err)
	}

	scheme := parsed.Scheme
	if scheme == "" {
		return nil, fmt.Errorf("scheduler: missing scheme in uri %q", uri)
	}

	mu.RLock()
	driver, ok := drivers[scheme]
	mu.RUnlock()
	if !ok {
		return nil, fmt.Errorf("scheduler: unknown driver %q (forgotten import?); registered drivers: %v", scheme, Drivers())
	}

	return driver.Open(ctx, parsed)
}
