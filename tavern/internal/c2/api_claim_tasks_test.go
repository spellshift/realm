package c2_test

import (
	"context"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/testing/protocmp"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/c2/c2test"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/beacon"
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
	tests := []struct {
		name     string
		req      *c2pb.ClaimTasksRequest
		wantResp *c2pb.ClaimTasksResponse
		wantCode codes.Code

		wantBeaconExist            bool
		wantBeaconLastSeenAtBefore time.Time
		wantBeaconLastSeenAtAfter  time.Time
	}{
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
					AvailableTransports: &c2pb.AvailableTransports{
						Transports: []*c2pb.Transport{
							{
								Uri:      "grpc://127.0.0.1:8080",
								Interval: uint64(60),
								Type:     c2pb.Transport_TRANSPORT_GRPC,
							},
						},
						ActiveIndex: 0,
					},
				},
			},
			wantResp: &c2pb.ClaimTasksResponse{},
			wantCode: codes.OK,

			wantBeaconExist:            true,
			wantBeaconLastSeenAtBefore: time.Now().UTC().Add(10 * time.Second),
			wantBeaconLastSeenAtAfter:  time.Now().UTC().Add(-10 * time.Second),
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
					AvailableTransports: &c2pb.AvailableTransports{
						Transports: []*c2pb.Transport{
							{
								Uri:      "grpc://127.0.0.1:8080",
								Interval: uint64(100),
								Type:     c2pb.Transport_TRANSPORT_GRPC,
							},
						},
						ActiveIndex: 0,
					},
				},
			},
			wantResp: &c2pb.ClaimTasksResponse{},
			wantCode: codes.OK,

			wantBeaconExist:            true,
			wantBeaconLastSeenAtBefore: time.Now().UTC().Add(10 * time.Second),
			wantBeaconLastSeenAtAfter:  time.Now().UTC().Add(-10 * time.Second),
		},
		{
			name: "Existing_Beacon",
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
					AvailableTransports: &c2pb.AvailableTransports{
						Transports: []*c2pb.Transport{
							{
								Uri:      "grpc://127.0.0.1:8080",
								Interval: uint64(100),
								Type:     c2pb.Transport_TRANSPORT_GRPC,
							},
						},
						ActiveIndex: 0,
					},
				},
			},
			wantResp: &c2pb.ClaimTasksResponse{
				Tasks: []*c2pb.Task{
					c2test.ConvertTaskToC2PB(t, ctx, existingTasks[0]),
					c2test.ConvertTaskToC2PB(t, ctx, existingTasks[1]),
				},
			},
			wantCode: codes.OK,

			wantBeaconExist:            true,
			wantBeaconLastSeenAtBefore: time.Now().UTC().Add(10 * time.Second),
			wantBeaconLastSeenAtAfter:  time.Now().UTC().Add(-10 * time.Second),
		},
	}

	// Run Tests
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			// Callback
			resp, err := client.ClaimTasks(ctx, tc.req)

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

			// Load Beacon
			testBeacon, err := graph.Beacon.Query().
				Where(
					beacon.Identifier(tc.req.Beacon.Identifier),
				).Only(ctx)
			if ent.IsNotFound(err) && !tc.wantBeaconExist {
				return
			}
			if err != nil {
				t.Errorf("failed to load beacon: %v", err)
				return
			}

			// Beacon Assertions
			assert.WithinRange(t, testBeacon.LastSeenAt, tc.wantBeaconLastSeenAtAfter, tc.wantBeaconLastSeenAtBefore)
		})
	}
}
