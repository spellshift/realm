package mem

import (
	"context"
	"fmt"
	"io"
	"log/slog"
	"net/http"
	"net/url"
	"strings"
	"sync"

	"github.com/robfig/cron/v3"
	"realm.pub/tavern/internal/scheduler"
)

func init() {
	scheduler.Register("mem", &Driver{})
}

// Driver implements scheduler.Driver for in-memory scheduling.
type Driver struct{}

// Open returns a new in-memory Scheduler. The URI is not used.
func (d *Driver) Open(_ context.Context, _ *url.URL) (scheduler.Scheduler, error) {
	c := cron.New()
	c.Start()
	return &Scheduler{
		cron: c,
		jobs: make(map[string]cron.EntryID),
	}, nil
}

// Scheduler is an in-memory implementation of scheduler.Scheduler backed by robfig/cron.
type Scheduler struct {
	mu   sync.Mutex
	cron *cron.Cron
	jobs map[string]cron.EntryID
}

// Schedule creates a job that fires on the given cron schedule.
// It returns an error if a job with the same name already exists.
func (s *Scheduler) Schedule(_ context.Context, job scheduler.Job) error {
	s.mu.Lock()
	defer s.mu.Unlock()

	if _, exists := s.jobs[job.Name]; exists {
		return fmt.Errorf("scheduler/mem: job %q already exists", job.Name)
	}

	target := job.HTTPTarget
	id, err := s.cron.AddFunc(job.Schedule, func() {
		s.fireHTTP(job.Name, target)
	})
	if err != nil {
		return fmt.Errorf("scheduler/mem: invalid cron expression: %w", err)
	}
	s.jobs[job.Name] = id
	return nil
}

// Close stops the cron scheduler and releases resources.
func (s *Scheduler) Close() error {
	s.cron.Stop()
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

	req, err := http.NewRequest(method, target.URL, body)
	if err != nil {
		slog.Error("scheduler/mem: failed to create request", "job", name, "error", err)
		return
	}
	for k, v := range target.Headers {
		req.Header.Set(k, v)
	}

	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		slog.Error("scheduler/mem: request failed", "job", name, "error", err)
		return
	}
	defer resp.Body.Close()

	if resp.StatusCode >= 400 {
		slog.Warn("scheduler/mem: non-success response", "job", name, "status", resp.StatusCode)
	}
}
