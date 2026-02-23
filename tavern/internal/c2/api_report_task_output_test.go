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
	"google.golang.org/protobuf/types/known/timestamppb"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/c2/c2test"
	"realm.pub/tavern/internal/ent"
)

func TestReportTaskOutput(t *testing.T) {
	// Setup Dependencies
	ctx := context.Background()
	client, graph, close, token := c2test.New(t)
	defer close()

	// Test Data
	now := timestamppb.Now()
	finishedAt := timestamppb.New(time.Now().UTC().Add(10 * time.Minute))
	existingBeacon := c2test.NewRandomBeacon(ctx, graph)
	existingTasks := []*ent.Task{
		c2test.NewRandomAssignedTask(ctx, graph, existingBeacon.Identifier),
		c2test.NewRandomAssignedTask(ctx, graph, existingBeacon.Identifier),
	}

	// Test Cases
	tests := []struct {
		name               string
		req                *c2pb.ReportTaskOutputRequest
		wantResp           *c2pb.ReportTaskOutputResponse
		wantCode           codes.Code
		wantOutput         string
		wantError          string
		wantExecStartedAt  *timestamppb.Timestamp
		wantExecFinishedAt *timestamppb.Timestamp
	}{
		{
			name: "First_Output",
			req: &c2pb.ReportTaskOutputRequest{
				Output: &c2pb.TaskOutput{
					Id:            int64(existingTasks[0].ID),
					Output:        "TestOutput",
					ExecStartedAt: now,
				},
			},
			wantResp:          &c2pb.ReportTaskOutputResponse{},
			wantCode:          codes.OK,
			wantOutput:        "TestOutput",
			wantExecStartedAt: now,
		},
		{
			name: "First_error",
			req: &c2pb.ReportTaskOutputRequest{
				Output: &c2pb.TaskOutput{
					Id:     int64(existingTasks[0].ID),
					Output: "",
					Error: &c2pb.TaskError{
						Msg: "hello error!",
					},
					ExecStartedAt: now,
				},
			},
			wantResp:          &c2pb.ReportTaskOutputResponse{},
			wantCode:          codes.OK,
			wantOutput:        "TestOutput",
			wantError:         "hello error!",
			wantExecStartedAt: now,
		},
		{
			name: "Append_Output",
			req: &c2pb.ReportTaskOutputRequest{
				Output: &c2pb.TaskOutput{
					Id:     int64(existingTasks[0].ID),
					Output: "_AppendedOutput",
					Error: &c2pb.TaskError{
						Msg: "_AppendEror",
					},
				},
			},
			wantResp:          &c2pb.ReportTaskOutputResponse{},
			wantCode:          codes.OK,
			wantOutput:        "TestOutput_AppendedOutput",
			wantError:         "hello error!_AppendEror",
			wantExecStartedAt: now,
		},
		{
			name: "Exec_Finished",
			req: &c2pb.ReportTaskOutputRequest{
				Output: &c2pb.TaskOutput{
					Id:             int64(existingTasks[0].ID),
					ExecFinishedAt: finishedAt,
				},
			},
			wantResp:           &c2pb.ReportTaskOutputResponse{},
			wantCode:           codes.OK,
			wantOutput:         "TestOutput_AppendedOutput",
			wantError:          "hello error!_AppendEror",
			wantExecStartedAt:  now,
			wantExecFinishedAt: finishedAt,
		},
		{
			name: "Not_Found",
			req: &c2pb.ReportTaskOutputRequest{
				Output: &c2pb.TaskOutput{
					Id: 999888777666,
				},
			},
			wantResp: nil,
			wantCode: codes.NotFound,
		},
		{
			name: "Invalid_Argument",
			req: &c2pb.ReportTaskOutputRequest{
				Output: &c2pb.TaskOutput{},
			},
			wantResp: nil,
			wantCode: codes.InvalidArgument,
		},
	}

	// Run Tests
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			// Callback
			// Ensure JWT present in request context
			if tc.req.Context == nil {
				tc.req.Context = &c2pb.TaskContext{Jwt: token}
			} else {
				tc.req.Context.Jwt = token
			}

			resp, err := client.ReportTaskOutput(ctx, tc.req)

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

			// Load Task

			testTask, err := graph.Task.Get(ctx, int(tc.req.Output.Id))
			require.NoError(t, err)

			// Task Assertions
			assert.Equal(t, tc.wantOutput, testTask.Output)
			assert.Equal(t, tc.wantError, testTask.Error)
		})
	}

}
