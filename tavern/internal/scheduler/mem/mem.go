package mem

import (
	"context"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strings"
	"sync"
	"time"

	"github.com/robfig/cron/v3"
	"realm.pub/tavern/internal/logging"
	"realm.pub/tavern/internal/scheduler"
)

var log = logging.For("scheduler.mem")

func init() {
	scheduler.Register("mem", &Driver{})
}

// Driver implements scheduler.Driver for in-memory scheduling.
type Driver struct{}

// Open returns a new in-memory Scheduler. The URI is not used.
func (d *Driver) Open(ctx context.Context, _ *url.URL) (scheduler.Scheduler, error) {
	c := cron.New()
	c.Start()
	log.DebugContext(ctx, "started in-memory scheduler")
	return &Scheduler{
		cron:   c,
		jobs:   make(map[string]cron.EntryID),
		timers: make(map[string]*time.Timer),
	}, nil
}

// Scheduler is an in-memory implementation of scheduler.Scheduler backed by robfig/cron.
type Scheduler struct {
	mu     sync.Mutex
	cron   *cron.Cron
	jobs   map[string]cron.EntryID
	timers map[string]*time.Timer
}

// Schedule creates a job that fires on the given cron schedule.
// It returns an error if a job with the same name already exists.
func (s *Scheduler) Schedule(ctx context.Context, job scheduler.Job) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	if _, exists := s.jobs[job.Name]; exists {
		return fmt.Errorf("scheduler/mem: %w: %s", scheduler.ErrJobExists, job.Name)
	}

	target := job.HTTPTarget
	id, err := s.cron.AddFunc(job.Schedule, func() {
		s.fireHTTP(job.Name, target)
	})
	if err != nil {
		return fmt.Errorf("scheduler/mem: invalid cron expression: %w", err)
	}
	s.jobs[job.Name] = id
	log.DebugContext(ctx, "scheduled cron job", "job", job.Name, "schedule", job.Schedule, "url", job.HTTPTarget.URL)
	return nil
}

// ScheduleAt creates a one-time job that fires at the given time.
// It returns an error if a job with the same name already exists.
func (s *Scheduler) ScheduleAt(ctx context.Context, job scheduler.OnceJob) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	if _, exists := s.jobs[job.Name]; exists {
		return fmt.Errorf("scheduler/mem: %w: %s", scheduler.ErrJobExists, job.Name)
	}
	if _, exists := s.timers[job.Name]; exists {
		return fmt.Errorf("scheduler/mem: %w: %s", scheduler.ErrJobExists, job.Name)
	}

	delay := time.Until(job.At)
	if delay < 0 {
		delay = 0
	}

	target := job.HTTPTarget
	name := job.Name
	timer := time.AfterFunc(delay, func() {
		s.fireHTTP(name, target)

		s.mu.Lock()
		delete(s.timers, name)
		s.mu.Unlock()
	})
	s.timers[job.Name] = timer
	log.DebugContext(ctx, "scheduled one-time job", "job", job.Name, "at", job.At.UTC(), "url", job.HTTPTarget.URL)
	return nil
}

// Close stops the cron scheduler, cancels pending timers, and releases resources.
func (s *Scheduler) Close() error {
	s.mu.Lock()
	defer s.mu.Unlock()

	s.cron.Stop()
	for _, timer := range s.timers {
		timer.Stop()
	}
	s.timers = make(map[string]*time.Timer)
	return nil
}

// fireHTTP performs the HTTP request defined by the job's target.
func (s *Scheduler) fireHTTP(name string, target scheduler.HTTPTarget) {
	method := target.Method
	if method == "" {
		method = http.MethodPost
	}

	var body io.Reader
	if len(target.Body) > 0 {
		body = strings.NewReader(string(target.Body))
	}

	log.Trace("firing scheduled job", "job", name, "method", method, "url", target.URL)

	req, err := http.NewRequest(method, target.URL, body)
	if err != nil {
		log.Error("failed to create request", "job", name, "error", err)
		return
	}
	for k, v := range target.Headers {
		req.Header.Set(k, v)
	}

	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		log.Error("scheduled job request failed", "job", name, "error", err)
		return
	}
	defer resp.Body.Close()

	if resp.StatusCode >= 400 {
		log.Warn("scheduled job returned non-success status", "job", name, "status", resp.StatusCode)
		return
	}
	log.Trace("scheduled job completed", "job", name, "status", resp.StatusCode)
}
