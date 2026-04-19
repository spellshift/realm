package mem

import (
	"context"
	"fmt"
	"net/http"
	"net/http/httptest"
	"sync/atomic"
	"testing"
	"time"

	"realm.pub/tavern/internal/scheduler"
)

func TestSchedule(t *testing.T) {
	var called atomic.Int32
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		called.Add(1)
		w.WriteHeader(http.StatusOK)
	}))
	defer srv.Close()

	d := &Driver{}
	s, err := d.Open(context.Background(), nil)
	if err != nil {
		t.Fatalf("Open: %v", err)
	}
	defer s.Close()

	err = s.Schedule(context.Background(), scheduler.Job{
		Name:     "test-job",
		Schedule: "@every 1s",
		HTTPTarget: scheduler.HTTPTarget{
			URL:    srv.URL,
			Method: http.MethodPost,
		},
	})
	if err != nil {
		t.Fatalf("Schedule: %v", err)
	}

	// Wait for at least one invocation
	deadline := time.Now().Add(5 * time.Second)
	for time.Now().Before(deadline) {
		if called.Load() > 0 {
			return
		}
		time.Sleep(100 * time.Millisecond)
	}
	t.Fatal("expected at least one HTTP call within 5s")
}

func TestScheduleDuplicate(t *testing.T) {
	d := &Driver{}
	s, err := d.Open(context.Background(), nil)
	if err != nil {
		t.Fatalf("Open: %v", err)
	}
	defer s.Close()

	job := scheduler.Job{
		Name:     "dup-job",
		Schedule: "@every 1h",
		HTTPTarget: scheduler.HTTPTarget{
			URL: "http://localhost:9999",
		},
	}

	if err := s.Schedule(context.Background(), job); err != nil {
		t.Fatalf("first Schedule: %v", err)
	}
	if err := s.Schedule(context.Background(), job); err == nil {
		t.Fatal("expected error on duplicate schedule")
	}
}

func TestScheduleInvalidCron(t *testing.T) {
	d := &Driver{}
	s, err := d.Open(context.Background(), nil)
	if err != nil {
		t.Fatalf("Open: %v", err)
	}
	defer s.Close()

	err = s.Schedule(context.Background(), scheduler.Job{
		Name:     "bad-cron",
		Schedule: "not-a-cron",
		HTTPTarget: scheduler.HTTPTarget{
			URL: "http://localhost:9999",
		},
	})
	if err == nil {
		t.Fatal("expected error for invalid cron expression")
	}
}

func TestClose(t *testing.T) {
	d := &Driver{}
	s, err := d.Open(context.Background(), nil)
	if err != nil {
		t.Fatalf("Open: %v", err)
	}
	if err := s.Close(); err != nil {
		t.Fatalf("Close: %v", err)
	}
}

func TestDriverRegistered(t *testing.T) {
	for _, name := range scheduler.Drivers() {
		if name == "mem" {
			return
		}
	}
	t.Fatal("mem driver not registered")
}

func TestScheduleMultipleJobs(t *testing.T) {
	d := &Driver{}
	s, err := d.Open(context.Background(), nil)
	if err != nil {
		t.Fatalf("Open: %v", err)
	}
	defer s.Close()

	for i := 0; i < 5; i++ {
		err := s.Schedule(context.Background(), scheduler.Job{
			Name:     fmt.Sprintf("job-%d", i),
			Schedule: "@every 1h",
			HTTPTarget: scheduler.HTTPTarget{
				URL: "http://localhost:9999",
			},
		})
		if err != nil {
			t.Fatalf("Schedule job-%d: %v", i, err)
		}
	}
}
