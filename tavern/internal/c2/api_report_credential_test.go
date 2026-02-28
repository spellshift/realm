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

func TestReportCredential(t *testing.T) {
	// Setup Dependencies
	client, graph, close, token := c2test.New(t)
	defer close()
    ctx := context.Background()

	// Test Data
	existingBeacon := c2test.NewRandomBeacon(ctx, graph)
	existingTask := c2test.NewRandomAssignedTask(ctx, graph, existingBeacon.Identifier)
	existingHost := existingBeacon.QueryHost().OnlyX(ctx)
	existingCredential := graph.HostCredential.Create().
		SetHost(existingHost).
		SetTask(existingTask).
		SetPrincipal("test-cred").
		SetSecret("test-secret").
		SetKind(epb.Credential_KIND_PASSWORD).
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
				Context: &c2pb.ReportCredentialRequest_TaskContext{
                    TaskContext: &c2pb.TaskContext{TaskId: int64(existingTask.ID), Jwt: token},
                },
				Credential: &epb.Credential{
					Principal: existingCredential.Principal,
					Secret:    existingCredential.Secret,
					Kind:      epb.Credential_KIND_PASSWORD,
				},
			},
			wantResp: &c2pb.ReportCredentialResponse{},
			wantCode: codes.OK,
			wantHostCredentials: []*epb.Credential{
				{
					Principal: existingCredential.Principal,
					Secret:    existingCredential.Secret,
					Kind:      epb.Credential_KIND_PASSWORD,
				},
				{
					Principal: existingCredential.Principal,
					Secret:    existingCredential.Secret,
					Kind:      epb.Credential_KIND_PASSWORD,
				},
			},
		},
		{
			name: "NewCredential",
			host: existingHost,
			task: existingTask,
			req: &c2pb.ReportCredentialRequest{
				Context: &c2pb.ReportCredentialRequest_TaskContext{
                    TaskContext: &c2pb.TaskContext{TaskId: int64(existingTask.ID), Jwt: token},
                },
				Credential: &epb.Credential{
					Principal: "root",
					Secret:    "changeme123",
					Kind:      epb.Credential_KIND_PASSWORD,
				},
			},
			wantResp: &c2pb.ReportCredentialResponse{},
			wantCode: codes.OK,
			wantHostCredentials: []*epb.Credential{
				{
					Principal: existingCredential.Principal,
					Secret:    existingCredential.Secret,
					Kind:      epb.Credential_KIND_PASSWORD,
				},
				{
					Principal: existingCredential.Principal,
					Secret:    existingCredential.Secret,
					Kind:      epb.Credential_KIND_PASSWORD,
				},
				{
					Principal: "root",
					Secret:    "changeme123",
					Kind:      epb.Credential_KIND_PASSWORD,
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
					Kind:      epb.Credential_KIND_UNSPECIFIED,
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
				Context: &c2pb.ReportCredentialRequest_TaskContext{
                    TaskContext: &c2pb.TaskContext{TaskId: int64(existingTask.ID), Jwt: token},
                },
			},
			wantResp: nil,
			wantCode: codes.InvalidArgument,
		},
		{
			name: "NotFound",
			req: &c2pb.ReportCredentialRequest{
				Context: &c2pb.ReportCredentialRequest_TaskContext{
                    TaskContext: &c2pb.TaskContext{TaskId: 99888777776666, Jwt: token},
                },
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
			st, _ := status.FromError(err)
			require.Equal(t, tc.wantCode.String(), st.Code().String(), err)
			if st.Code() != codes.OK {
				// Do not continue if we expected error code
				return
			}

			// Assert Response
			if diff := cmp.Diff(tc.wantResp, resp, protocmp.Transform()); diff != "" {
				t.Errorf("invalid response (-want +got): %v", diff)
			}

			// Reload Host
            if tc.host != nil {
			    host := graph.Host.GetX(ctx, tc.host.ID)

			    // Assert Host Credentials
			    var pbHostCreds []*epb.Credential
			    entHostCredentials := host.QueryCredentials().AllX(ctx)
			    for _, cred := range entHostCredentials {
				    pbHostCreds = append(pbHostCreds, &epb.Credential{Principal: cred.Principal, Secret: cred.Secret, Kind: cred.Kind})
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
            }
		})
	}
}
