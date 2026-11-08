package gcp

import (
	"context"
	"errors"
	"fmt"
	"log/slog"
	"net/url"
	"strings"

	cloudscheduler "cloud.google.com/go/scheduler/apiv1"
	schedulerpb "cloud.google.com/go/scheduler/apiv1/schedulerpb"
	"google.golang.org/api/iterator"
	"google.golang.org/api/option"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/types/known/fieldmaskpb"
	"realm.pub/tavern/internal/scheduler"
)

func init() {
	scheduler.Register("gcp", &Driver{})
}

// jobIterator abstracts iteration over Cloud Scheduler jobs.
type jobIterator interface {
	Next() (*schedulerpb.Job, error)
}

// cloudClient abstracts the GCP Cloud Scheduler API for testability.
type cloudClient interface {
	CreateJob(ctx context.Context, req *schedulerpb.CreateJobRequest) (*schedulerpb.Job, error)
	UpdateJob(ctx context.Context, req *schedulerpb.UpdateJobRequest) (*schedulerpb.Job, error)
	PauseJob(ctx context.Context, req *schedulerpb.PauseJobRequest) (*schedulerpb.Job, error)
	ResumeJob(ctx context.Context, req *schedulerpb.ResumeJobRequest) (*schedulerpb.Job, error)
	ListJobs(ctx context.Context, req *schedulerpb.ListJobsRequest) jobIterator
	Close() error
}

// gcpClient wraps the real GCP Cloud Scheduler client to satisfy cloudClient.
type gcpClient struct {
	inner *cloudscheduler.CloudSchedulerClient
}

func (c *gcpClient) CreateJob(ctx context.Context, req *schedulerpb.CreateJobRequest) (*schedulerpb.Job, error) {
	return c.inner.CreateJob(ctx, req)
}

func (c *gcpClient) UpdateJob(ctx context.Context, req *schedulerpb.UpdateJobRequest) (*schedulerpb.Job, error) {
	return c.inner.UpdateJob(ctx, req)
}

func (c *gcpClient) PauseJob(ctx context.Context, req *schedulerpb.PauseJobRequest) (*schedulerpb.Job, error) {
	return c.inner.PauseJob(ctx, req)
}

func (c *gcpClient) ResumeJob(ctx context.Context, req *schedulerpb.ResumeJobRequest) (*schedulerpb.Job, error) {
	return c.inner.ResumeJob(ctx, req)
}

func (c *gcpClient) ListJobs(ctx context.Context, req *schedulerpb.ListJobsRequest) jobIterator {
	return c.inner.ListJobs(ctx, req)
}

func (c *gcpClient) Close() error {
	return c.inner.Close()
}

// Driver implements scheduler.Driver for Google Cloud Scheduler.
type Driver struct{}

// Open creates a new GCP Cloud Scheduler client.
// The URI should be of the form:
//
//	gcp://projects/{project}/locations/{location}
//
// Query parameters are passed as client options (currently unused but reserved).
func (d *Driver) Open(ctx context.Context, uri *url.URL) (scheduler.Scheduler, error) {
	// Reconstruct the parent resource name from the URI host + path.
	// url.Parse of "gcp://projects/my-proj/locations/us-central1" gives:
	//   Host = "projects"
	//   Path = "/my-proj/locations/us-central1"
	parent := uri.Host + uri.Path
	if parent == "" {
		return nil, fmt.Errorf("scheduler/gcp: URI must specify a parent (e.g. gcp://projects/{project}/locations/{location})")
	}

	slog.InfoContext(ctx, "scheduler/gcp: creating Cloud Scheduler client", "parent", parent)

	var opts []option.ClientOption
	client, err := cloudscheduler.NewCloudSchedulerClient(ctx, opts...)
	if err != nil {
		return nil, fmt.Errorf("scheduler/gcp: failed to create client (parent=%q): %w", parent, err)
	}

	slog.InfoContext(ctx, "scheduler/gcp: client created successfully", "parent", parent)

	return &Scheduler{
		client: &gcpClient{inner: client},
		parent: parent,
	}, nil
}

// Scheduler is a GCP Cloud Scheduler implementation of scheduler.Scheduler.
type Scheduler struct {
	client cloudClient
	parent string // e.g. "projects/my-proj/locations/us-central1"
}

// Schedule creates a Cloud Scheduler job.
// It first checks whether a job with the same name already exists, returning an
// error if so. This makes the method safe for use in a distributed system where
// multiple replicas might attempt to create the same job.
func (s *Scheduler) Schedule(ctx context.Context, job scheduler.Job) error {
	fullName := fmt.Sprintf("%s/jobs/%s", s.parent, job.Name)

	slog.InfoContext(ctx, "scheduler/gcp: scheduling job", "name", fullName, "schedule", job.Schedule, "url", job.HTTPTarget.URL, "method", job.HTTPTarget.Method)

	// Check for an existing job with the same name.
	if exists, err := s.jobExists(ctx, fullName); err != nil {
		return fmt.Errorf("scheduler/gcp: failed checking for existing job %q: %w", fullName, err)
	} else if exists {
		return fmt.Errorf("scheduler/gcp: %w: %s", scheduler.ErrJobExists, job.Name)
	}

	pbJob := recurringJobProto(job, fullName)

	if _, err := s.client.CreateJob(ctx, &schedulerpb.CreateJobRequest{
		Parent: s.parent,
		Job:    pbJob,
	}); err != nil {
		return fmt.Errorf("scheduler/gcp: failed to create job %q (schedule=%q, url=%q): %w", fullName, job.Schedule, job.HTTPTarget.URL, err)
	}

	slog.InfoContext(ctx, "scheduler/gcp: created job", "name", fullName, "schedule", job.Schedule, "url", job.HTTPTarget.URL)
	return nil
}

func recurringJobProto(job scheduler.Job, fullName string) *schedulerpb.Job {
	method := schedulerpb.HttpMethod_POST
	if job.HTTPTarget.Method != "" {
		method = httpMethodToProto(job.HTTPTarget.Method)
	}
	target := &schedulerpb.HttpTarget{
		Uri: job.HTTPTarget.URL, HttpMethod: method,
		Headers: job.HTTPTarget.Headers, Body: job.HTTPTarget.Body,
	}
	if job.HTTPTarget.OIDCServiceAccountEmail != "" {
		target.AuthorizationHeader = &schedulerpb.HttpTarget_OidcToken{OidcToken: &schedulerpb.OidcToken{
			ServiceAccountEmail: job.HTTPTarget.OIDCServiceAccountEmail,
			Audience:            job.HTTPTarget.OIDCAudience,
		}}
	}
	return &schedulerpb.Job{Name: fullName, Schedule: job.Schedule, TimeZone: "UTC", Target: &schedulerpb.Job_HttpTarget{HttpTarget: target}}
}

// Ensure creates or updates a recurring job with the current target and credentials.
func (s *Scheduler) Ensure(ctx context.Context, job scheduler.Job) error {
	fullName := fmt.Sprintf("%s/jobs/%s", s.parent, job.Name)
	existing, err := s.findJob(ctx, fullName)
	if err != nil {
		return fmt.Errorf("scheduler/gcp: check job %q: %w", fullName, err)
	}
	if existing == nil {
		err = s.Schedule(ctx, job)
		if err == nil {
			return nil
		}
		if !errors.Is(err, scheduler.ErrJobExists) && status.Code(err) != codes.AlreadyExists {
			return err
		}
	}
	_, err = s.client.UpdateJob(ctx, &schedulerpb.UpdateJobRequest{
		Job:        recurringJobProto(job, fullName),
		UpdateMask: &fieldmaskpb.FieldMask{Paths: []string{"schedule", "time_zone", "http_target"}},
	})
	if err != nil {
		return fmt.Errorf("scheduler/gcp: update job %q: %w", fullName, err)
	}
	if existing != nil && existing.GetState() == schedulerpb.Job_PAUSED {
		if _, err := s.client.ResumeJob(ctx, &schedulerpb.ResumeJobRequest{Name: fullName}); err != nil {
			return fmt.Errorf("scheduler/gcp: resume job %q: %w", fullName, err)
		}
	}
	return nil
}

// Disable pauses a recurring job without releasing its name.
func (s *Scheduler) Disable(ctx context.Context, name string) error {
	fullName := fmt.Sprintf("%s/jobs/%s", s.parent, name)
	existing, err := s.findJob(ctx, fullName)
	if err != nil {
		return fmt.Errorf("scheduler/gcp: check job %q: %w", fullName, err)
	}
	if existing == nil || existing.GetState() != schedulerpb.Job_ENABLED {
		return nil
	}
	if _, err := s.client.PauseJob(ctx, &schedulerpb.PauseJobRequest{Name: fullName}); err != nil && status.Code(err) != codes.NotFound {
		return fmt.Errorf("scheduler/gcp: pause job %q: %w", fullName, err)
	}
	return nil
}

// ScheduleAt creates a one-time Cloud Scheduler job that fires at the specified time.
//
// GCP Cloud Scheduler does not natively support one-shot jobs. This method
// converts the target time to a cron expression targeting the exact minute
// (e.g. "45 14 19 4 *" for April 19 at 14:45 UTC). The resulting job will
// repeat annually at the same date and time; callers should ensure their
// HTTP handler is idempotent.
//
// If the target time is in the past, the cron expression will still be set
// for the exact minute; Cloud Scheduler will fire the job at the next
// occurrence of that cron expression.
func (s *Scheduler) ScheduleAt(ctx context.Context, job scheduler.OnceJob) error {
	fullName := fmt.Sprintf("%s/jobs/%s", s.parent, job.Name)

	slog.InfoContext(ctx, "scheduler/gcp: scheduling one-time job", "name", fullName, "at", job.At.UTC(), "url", job.HTTPTarget.URL, "method", job.HTTPTarget.Method)

	// Check for an existing job with the same name.
	if exists, err := s.jobExists(ctx, fullName); err != nil {
		return fmt.Errorf("scheduler/gcp: failed checking for existing job %q: %w", fullName, err)
	} else if exists {
		return fmt.Errorf("scheduler/gcp: %w: %s", scheduler.ErrJobExists, job.Name)
	}

	method := schedulerpb.HttpMethod_POST
	if job.HTTPTarget.Method != "" {
		method = httpMethodToProto(job.HTTPTarget.Method)
	}

	// Convert the target time to a cron expression in UTC.
	utc := job.At.UTC()
	schedule := fmt.Sprintf("%d %d %d %d *", utc.Minute(), utc.Hour(), utc.Day(), utc.Month())

	pbJob := &schedulerpb.Job{
		Name:     fullName,
		Schedule: schedule,
		TimeZone: "UTC",
		Target: &schedulerpb.Job_HttpTarget{
			HttpTarget: &schedulerpb.HttpTarget{
				Uri:        job.HTTPTarget.URL,
				HttpMethod: method,
				Headers:    job.HTTPTarget.Headers,
				Body:       job.HTTPTarget.Body,
			},
		},
	}

	if _, err := s.client.CreateJob(ctx, &schedulerpb.CreateJobRequest{
		Parent: s.parent,
		Job:    pbJob,
	}); err != nil {
		return fmt.Errorf("scheduler/gcp: failed to create one-time job %q (schedule=%q, url=%q): %w", fullName, schedule, job.HTTPTarget.URL, err)
	}

	slog.InfoContext(ctx, "scheduler/gcp: created one-time job", "name", fullName, "schedule", schedule, "target_time", utc, "url", job.HTTPTarget.URL)
	return nil
}

// Close releases the underlying GCP client connection.
func (s *Scheduler) Close() error {
	return s.client.Close()
}

// jobExists returns true if a job with the given full resource name exists.
func (s *Scheduler) jobExists(ctx context.Context, fullName string) (bool, error) {
	job, err := s.findJob(ctx, fullName)
	return job != nil, err
}

func (s *Scheduler) findJob(ctx context.Context, fullName string) (*schedulerpb.Job, error) {
	it := s.client.ListJobs(ctx, &schedulerpb.ListJobsRequest{
		Parent: s.parent,
	})
	for {
		j, err := it.Next()
		if err == iterator.Done {
			break
		}
		if err != nil {
			return nil, err
		}
		if j.GetName() == fullName {
			return j, nil
		}
	}
	return nil, nil
}

// httpMethodToProto maps an HTTP method string to the protobuf enum.
func httpMethodToProto(method string) schedulerpb.HttpMethod {
	switch strings.ToUpper(method) {
	case "GET":
		return schedulerpb.HttpMethod_GET
	case "POST":
		return schedulerpb.HttpMethod_POST
	case "PUT":
		return schedulerpb.HttpMethod_PUT
	case "DELETE":
		return schedulerpb.HttpMethod_DELETE
	case "PATCH":
		return schedulerpb.HttpMethod_PATCH
	case "HEAD":
		return schedulerpb.HttpMethod_HEAD
	case "OPTIONS":
		return schedulerpb.HttpMethod_OPTIONS
	default:
		return schedulerpb.HttpMethod_HTTP_METHOD_UNSPECIFIED
	}
}
