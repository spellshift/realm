package c2_test

import (
	"context"
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/testing/protocmp"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/c2/c2test"
	"realm.pub/tavern/internal/ent"
)

func TestClaimTasks(t *testing.T) {
	// Setup Dependencies
	ctx := context.Background()
	client, graph, close := c2test.New(t)
	defer close()

	// Test Data
	existingBeacon := c2test.NewRandomBeacon(ctx, graph)
	existingTasks := []*ent.Task{
		c2test.NewRandomAssignedTask(ctx, graph, existingBeacon.Identifier),
		c2test.NewRandomAssignedTask(ctx, graph, existingBeacon.Identifier),
	}

	// Test Cases
	type testCase struct {
		name     string
		req      *c2pb.ClaimTasksRequest
		wantResp *c2pb.ClaimTasksResponse
		wantCode codes.Code
	}
	tests := []testCase{
		{
			name: "First_Callback",
			req: &c2pb.ClaimTasksRequest{
				Beacon: &c2pb.Beacon{
					Identifier: "test-beacon-001",
					Principal:  "root",
					Agent: &c2pb.Agent{
						Identifier: "test-agent",
					},
					Host: &c2pb.Host{
						Identifier: "test-host",
						Name:       "host-for-test",
						Platform:   c2pb.Host_PLATFORM_LINUX,
						PrimaryIp:  "127.0.0.1",
					},
					Interval: uint64(100),
				},
			},
			wantResp: &c2pb.ClaimTasksResponse{},
			wantCode: codes.OK,
		},
		{
			name: "Second_Callback",
			req: &c2pb.ClaimTasksRequest{
				Beacon: &c2pb.Beacon{
					Identifier: "test-beacon-001",
					Principal:  "root",
					Agent: &c2pb.Agent{
						Identifier: "test-agent",
					},
					Host: &c2pb.Host{
						Identifier: "test-host",
						Name:       "host-for-test",
						Platform:   c2pb.Host_PLATFORM_LINUX,
						PrimaryIp:  "127.0.0.1",
					},
					Interval: uint64(100),
				},
			},
			wantResp: &c2pb.ClaimTasksResponse{},
			wantCode: codes.OK,
		},
		{
			name: "Callback_With_Tasks",
			req: &c2pb.ClaimTasksRequest{
				Beacon: &c2pb.Beacon{
					Identifier: existingBeacon.Identifier,
					Principal:  "root",
					Agent: &c2pb.Agent{
						Identifier: "test-agent",
					},
					Host: &c2pb.Host{
						Identifier: "test-host",
						Name:       "host-for-test",
						Platform:   c2pb.Host_PLATFORM_LINUX,
						PrimaryIp:  "127.0.0.1",
					},
					Interval: uint64(100),
				},
			},
			wantResp: &c2pb.ClaimTasksResponse{
				Tasks: []*c2pb.Task{
					c2test.ConvertTaskToC2PB(t, ctx, existingTasks[0]),
					c2test.ConvertTaskToC2PB(t, ctx, existingTasks[1]),
				},
			},
			wantCode: codes.OK,
		},
	}

	testHandler := func(t *testing.T, tc testCase) {
		resp, err := client.ClaimTasks(ctx, tc.req)
		require.Equal(t, tc.wantCode.String(), status.Code(err).String(), err)
		if status.Code(err) != codes.OK {
			// Do not continue if we expected error code
			return
		}

		if diff := cmp.Diff(tc.wantResp, resp, protocmp.Transform()); diff != "" {
			t.Errorf("invalid response (-want +got): %v", diff)
		}

	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			testHandler(t, tc)
		})
	}
}
