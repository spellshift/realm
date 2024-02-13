package c2_test

import (
	"context"
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/testing/protocmp"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/c2/c2test"
	"realm.pub/tavern/internal/c2/epb"
	"realm.pub/tavern/internal/ent"
)

func TestReportCredentialList(t *testing.T) {
	// Setup Dependencies
	ctx := context.Background()
	client, graph, close := c2test.New(t)
	defer close()

	// Test Data
	existingBeacon := c2test.NewRandomBeacon(ctx, graph)
	existingTask := c2test.NewRandomAssignedTask(ctx, graph, existingBeacon.Identifier)
	existingHost := existingBeacon.QueryHost().OnlyX(ctx)
	existingCredential := graph.HostCredential.Create().
		SetHost(existingHost).
		SetTask(existingTask).
		SetPrincipal("test-cred").
		SetSecret("test-secret").
		SaveX(ctx)

	// Test Cases
	tests := []struct {
		name string
		host *ent.Host
		task *ent.Task
		req  *c2pb.ReportCredentialRequest

		wantResp            *c2pb.ReportCredentialResponse
		wantCode            codes.Code
		wantHostCredentials []*epb.Credential
	}{
		{
			name: "DuplicateCredential",
			host: existingHost,
			task: existingTask,
			req: &c2pb.ReportCredentialRequest{
				TaskId: int64(existingTask.ID),
				Credential: &epb.Credential{
					Principal: existingCredential.Principal,
					Secret:    existingCredential.Secret,
				},
			},
			wantResp: &c2pb.ReportCredentialResponse{},
			wantCode: codes.OK,
			wantHostCredentials: []*epb.Credential{
				{
					Principal: existingCredential.Principal,
					Secret:    existingCredential.Secret,
				},
				{
					Principal: existingCredential.Principal,
					Secret:    existingCredential.Secret,
				},
			},
		},
		{
			name: "NewCredential",
			host: existingHost,
			task: existingTask,
			req: &c2pb.ReportCredentialRequest{
				TaskId: int64(existingTask.ID),
				Credential: &epb.Credential{
					Principal: "root",
					Secret:    "changeme123",
				},
			},
			wantResp: &c2pb.ReportCredentialResponse{},
			wantCode: codes.OK,
			wantHostCredentials: []*epb.Credential{
				{
					Principal: existingCredential.Principal,
					Secret:    existingCredential.Secret,
				},
				{
					Principal: existingCredential.Principal,
					Secret:    existingCredential.Secret,
				},
				{
					Principal: "root",
					Secret:    "changeme123",
				},
			},
		},

		{
			name: "NoTaskID",
			host: existingHost,
			task: existingTask,
			req: &c2pb.ReportCredentialRequest{
				Credential: &epb.Credential{
					Principal: "root",
					Secret:    "this_will_not_work",
				},
			},
			wantResp: nil,
			wantCode: codes.InvalidArgument,
		},
		{
			name: "NoCredential",
			host: existingHost,
			task: existingTask,
			req: &c2pb.ReportCredentialRequest{
				TaskId: int64(existingTask.ID),
			},
			wantResp: nil,
			wantCode: codes.InvalidArgument,
		},
		{
			name: "NotFound",
			req: &c2pb.ReportCredentialRequest{
				TaskId: 99888777776666,
				Credential: &epb.Credential{
					Principal: "root",
					Secret:    "oopsies",
				},
			},
			wantResp: nil,
			wantCode: codes.NotFound,
		},
	}

	// Run Tests
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			// gRPC
			resp, err := client.ReportCredential(ctx, tc.req)

			// Assert Response Code
			require.Equal(t, tc.wantCode.String(), status.Code(err).String(), err)
			if status.Code(err) != codes.OK {
				// Do not continue if we expected error code
				return
			}

			// Assert Response
			if diff := cmp.Diff(tc.wantResp, resp, protocmp.Transform()); diff != "" {
				t.Errorf("invalid response (-want +got): %v", diff)
			}

			// Reload Host
			host := graph.Host.GetX(ctx, tc.host.ID)

			// Assert Host Credentials
			var pbHostCreds []*epb.Credential
			entHostCredentials := host.QueryCredentials().AllX(ctx)
			for _, cred := range entHostCredentials {
				pbHostCreds = append(pbHostCreds, &epb.Credential{Principal: cred.Principal, Secret: cred.Secret})
			}

			comparer := func(x any, y any) bool {
				credX, okX := x.(*epb.Credential)
				credY, okY := y.(*epb.Credential)
				if !okX || !okY {
					return false
				}

				return credX.Principal < credY.Principal
			}
			assert.Equal(t, len(tc.wantHostCredentials), len(pbHostCreds))
			if diff := cmp.Diff(tc.wantHostCredentials, pbHostCreds, protocmp.Transform(), cmpopts.SortSlices(comparer)); diff != "" {
				t.Errorf("invalid host credentials (-want +got): %v", diff)
			}
		})
	}
}
