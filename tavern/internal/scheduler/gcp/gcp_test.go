package gcp

import (
	"context"
	"testing"

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
