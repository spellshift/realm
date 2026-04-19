package gcp

import (
	"context"
	"fmt"
	"testing"
	"time"

	schedulerpb "cloud.google.com/go/scheduler/apiv1/schedulerpb"
	"google.golang.org/api/iterator"
	"realm.pub/tavern/internal/scheduler"
)

// mockIterator is a test iterator that returns a fixed list of jobs.
type mockIterator struct {
	jobs []*schedulerpb.Job
	idx  int
}

func (m *mockIterator) Next() (*schedulerpb.Job, error) {
	if m.idx >= len(m.jobs) {
		return nil, iterator.Done
	}
	j := m.jobs[m.idx]
	m.idx++
	return j, nil
}

// mockClient is a test double for cloudClient.
type mockClient struct {
	jobs    []*schedulerpb.Job
	created []*schedulerpb.CreateJobRequest
}

func (m *mockClient) CreateJob(_ context.Context, req *schedulerpb.CreateJobRequest) (*schedulerpb.Job, error) {
	m.created = append(m.created, req)
	m.jobs = append(m.jobs, req.GetJob())
	return req.GetJob(), nil
}

func (m *mockClient) ListJobs(_ context.Context, _ *schedulerpb.ListJobsRequest) jobIterator {
	return &mockIterator{jobs: m.jobs}
}

func (m *mockClient) Close() error {
	return nil
}

func TestDriverRegistered(t *testing.T) {
	for _, name := range scheduler.Drivers() {
		if name == "gcp" {
			return
		}
	}
	t.Fatal("gcp driver not registered")
}

func TestHTTPMethodToProto(t *testing.T) {
	tests := []struct {
		method string
		want   schedulerpb.HttpMethod
	}{
		{"GET", schedulerpb.HttpMethod_GET},
		{"POST", schedulerpb.HttpMethod_POST},
		{"PUT", schedulerpb.HttpMethod_PUT},
		{"DELETE", schedulerpb.HttpMethod_DELETE},
		{"PATCH", schedulerpb.HttpMethod_PATCH},
		{"HEAD", schedulerpb.HttpMethod_HEAD},
		{"OPTIONS", schedulerpb.HttpMethod_OPTIONS},
		{"get", schedulerpb.HttpMethod_GET},
		{"", schedulerpb.HttpMethod_HTTP_METHOD_UNSPECIFIED},
		{"INVALID", schedulerpb.HttpMethod_HTTP_METHOD_UNSPECIFIED},
	}
	for _, tt := range tests {
		t.Run(tt.method, func(t *testing.T) {
			got := httpMethodToProto(tt.method)
			if got != tt.want {
				t.Errorf("httpMethodToProto(%q) = %v, want %v", tt.method, got, tt.want)
			}
		})
	}
}

func TestScheduleDuplicate(t *testing.T) {
	mock := &mockClient{}
	s := &Scheduler{
		client: mock,
		parent: "projects/test-proj/locations/us-central1",
	}

	job := scheduler.Job{
		Name:     "my-job",
		Schedule: "*/5 * * * *",
		HTTPTarget: scheduler.HTTPTarget{
			URL:    "https://example.com/callback",
			Method: "POST",
		},
	}

	// First schedule should succeed.
	if err := s.Schedule(context.Background(), job); err != nil {
		t.Fatalf("first Schedule: %v", err)
	}
	if len(mock.created) != 1 {
		t.Fatalf("expected 1 CreateJob call, got %d", len(mock.created))
	}

	// Second schedule of the same job should fail with a duplicate error.
	err := s.Schedule(context.Background(), job)
	if err == nil {
		t.Fatal("expected error on duplicate schedule, got nil")
	}
	if len(mock.created) != 1 {
		t.Fatalf("expected CreateJob not to be called again, got %d calls", len(mock.created))
	}
}

func TestScheduleCreatesJob(t *testing.T) {
	mock := &mockClient{}
	s := &Scheduler{
		client: mock,
		parent: "projects/test-proj/locations/us-central1",
	}

	job := scheduler.Job{
		Name:     "new-job",
		Schedule: "0 * * * *",
		HTTPTarget: scheduler.HTTPTarget{
			URL:    "https://example.com/run",
			Method: "GET",
			Headers: map[string]string{
				"Authorization": "Bearer token",
			},
			Body: []byte(`{"key":"value"}`),
		},
	}

	if err := s.Schedule(context.Background(), job); err != nil {
		t.Fatalf("Schedule: %v", err)
	}

	if len(mock.created) != 1 {
		t.Fatalf("expected 1 CreateJob call, got %d", len(mock.created))
	}

	req := mock.created[0]
	if req.Parent != "projects/test-proj/locations/us-central1" {
		t.Errorf("unexpected parent: %s", req.Parent)
	}
	pbJob := req.GetJob()
	if pbJob.GetName() != "projects/test-proj/locations/us-central1/jobs/new-job" {
		t.Errorf("unexpected job name: %s", pbJob.GetName())
	}
	if pbJob.GetSchedule() != "0 * * * *" {
		t.Errorf("unexpected schedule: %s", pbJob.GetSchedule())
	}
	httpTarget := pbJob.GetHttpTarget()
	if httpTarget.GetUri() != "https://example.com/run" {
		t.Errorf("unexpected URI: %s", httpTarget.GetUri())
	}
	if httpTarget.GetHttpMethod() != schedulerpb.HttpMethod_GET {
		t.Errorf("unexpected HTTP method: %v", httpTarget.GetHttpMethod())
	}
}

func TestScheduleAtCreatesJob(t *testing.T) {
	mock := &mockClient{}
	s := &Scheduler{
		client: mock,
		parent: "projects/test-proj/locations/us-central1",
	}

	targetTime := time.Date(2026, 4, 19, 14, 45, 0, 0, time.UTC)
	job := scheduler.OnceJob{
		Name: "once-job",
		At:   targetTime,
		HTTPTarget: scheduler.HTTPTarget{
			URL:    "https://example.com/host-check",
			Method: "POST",
			Headers: map[string]string{
				"Content-Type": "application/json",
			},
			Body: []byte(`{"host_id":42}`),
		},
	}

	if err := s.ScheduleAt(context.Background(), job); err != nil {
		t.Fatalf("ScheduleAt: %v", err)
	}

	if len(mock.created) != 1 {
		t.Fatalf("expected 1 CreateJob call, got %d", len(mock.created))
	}

	req := mock.created[0]
	if req.Parent != "projects/test-proj/locations/us-central1" {
		t.Errorf("unexpected parent: %s", req.Parent)
	}

	pbJob := req.GetJob()
	if pbJob.GetName() != "projects/test-proj/locations/us-central1/jobs/once-job" {
		t.Errorf("unexpected job name: %s", pbJob.GetName())
	}

	// Cron expression should match the target time in UTC: minute hour day month *
	wantSchedule := fmt.Sprintf("%d %d %d %d *", targetTime.Minute(), targetTime.Hour(), targetTime.Day(), targetTime.Month())
	if pbJob.GetSchedule() != wantSchedule {
		t.Errorf("unexpected schedule: got %q, want %q", pbJob.GetSchedule(), wantSchedule)
	}
	if pbJob.GetTimeZone() != "UTC" {
		t.Errorf("unexpected timezone: %s", pbJob.GetTimeZone())
	}

	httpTarget := pbJob.GetHttpTarget()
	if httpTarget.GetUri() != "https://example.com/host-check" {
		t.Errorf("unexpected URI: %s", httpTarget.GetUri())
	}
	if httpTarget.GetHttpMethod() != schedulerpb.HttpMethod_POST {
		t.Errorf("unexpected HTTP method: %v", httpTarget.GetHttpMethod())
	}
	if string(httpTarget.GetBody()) != `{"host_id":42}` {
		t.Errorf("unexpected body: %s", string(httpTarget.GetBody()))
	}
}

func TestScheduleAtDuplicate(t *testing.T) {
	mock := &mockClient{}
	s := &Scheduler{
		client: mock,
		parent: "projects/test-proj/locations/us-central1",
	}

	job := scheduler.OnceJob{
		Name: "dup-once-job",
		At:   time.Date(2026, 5, 1, 12, 0, 0, 0, time.UTC),
		HTTPTarget: scheduler.HTTPTarget{
			URL: "https://example.com/check",
		},
	}

	// First call should succeed.
	if err := s.ScheduleAt(context.Background(), job); err != nil {
		t.Fatalf("first ScheduleAt: %v", err)
	}
	if len(mock.created) != 1 {
		t.Fatalf("expected 1 CreateJob call, got %d", len(mock.created))
	}

	// Second call with the same name should fail.
	err := s.ScheduleAt(context.Background(), job)
	if err == nil {
		t.Fatal("expected error on duplicate ScheduleAt, got nil")
	}
	if len(mock.created) != 1 {
		t.Fatalf("expected CreateJob not to be called again, got %d calls", len(mock.created))
	}
}

func TestScheduleAtNonUTCTime(t *testing.T) {
	mock := &mockClient{}
	s := &Scheduler{
		client: mock,
		parent: "projects/test-proj/locations/us-central1",
	}

	// Use a non-UTC timezone; the cron expression should still be derived in UTC.
	loc, err := time.LoadLocation("America/New_York")
	if err != nil {
		t.Fatalf("failed to load timezone: %v", err)
	}
	// 10:30 AM EST = 15:30 UTC (during standard time)
	targetTime := time.Date(2026, 12, 15, 10, 30, 0, 0, loc)

	job := scheduler.OnceJob{
		Name: "tz-job",
		At:   targetTime,
		HTTPTarget: scheduler.HTTPTarget{
			URL: "https://example.com/check",
		},
	}

	if err := s.ScheduleAt(context.Background(), job); err != nil {
		t.Fatalf("ScheduleAt: %v", err)
	}

	pbJob := mock.created[0].GetJob()
	utc := targetTime.UTC()
	wantSchedule := fmt.Sprintf("%d %d %d %d *", utc.Minute(), utc.Hour(), utc.Day(), utc.Month())
	if pbJob.GetSchedule() != wantSchedule {
		t.Errorf("unexpected schedule: got %q, want %q", pbJob.GetSchedule(), wantSchedule)
	}
}

func TestScheduleAtDefaultMethod(t *testing.T) {
	mock := &mockClient{}
	s := &Scheduler{
		client: mock,
		parent: "projects/test-proj/locations/us-central1",
	}

	job := scheduler.OnceJob{
		Name: "default-method-job",
		At:   time.Date(2026, 6, 1, 0, 0, 0, 0, time.UTC),
		HTTPTarget: scheduler.HTTPTarget{
			URL: "https://example.com/check",
			// Method intentionally left empty — should default to POST.
		},
	}

	if err := s.ScheduleAt(context.Background(), job); err != nil {
		t.Fatalf("ScheduleAt: %v", err)
	}

	httpTarget := mock.created[0].GetJob().GetHttpTarget()
	if httpTarget.GetHttpMethod() != schedulerpb.HttpMethod_POST {
		t.Errorf("expected default POST method, got %v", httpTarget.GetHttpMethod())
	}
}
