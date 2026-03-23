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

func TestReportOutput(t *testing.T) {
	// Setup Dependencies
	client, graph, close, token := c2test.New(t)
	defer close()
	ctx := context.Background()

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
		req                *c2pb.ReportOutputRequest
		wantResp           *c2pb.ReportOutputResponse
		wantCode           codes.Code
		wantOutput         string
		wantError          string
		wantExecStartedAt  *timestamppb.Timestamp
		wantExecFinishedAt *timestamppb.Timestamp
		targetTaskID       int64 // Helper to know which task to check
	}{
		{
			name: "First_Output",
			req: &c2pb.ReportOutputRequest{
				Message: &c2pb.ReportOutputRequest_TaskOutput{
					TaskOutput: &c2pb.ReportTaskOutputMessage{
						Context: &c2pb.TaskContext{TaskId: int64(existingTasks[0].ID), Jwt: token},
						Output: &c2pb.TaskOutput{
							Id:            int64(existingTasks[0].ID),
							Output:        "TestOutput",
							ExecStartedAt: now,
						},
					},
				},
			},
			wantResp:          &c2pb.ReportOutputResponse{},
			wantCode:          codes.OK,
			wantOutput:        "TestOutput",
			wantExecStartedAt: now,
			targetTaskID:      int64(existingTasks[0].ID),
		},
		{
			name: "First_error",
			req: &c2pb.ReportOutputRequest{
				Message: &c2pb.ReportOutputRequest_TaskOutput{
					TaskOutput: &c2pb.ReportTaskOutputMessage{
						Context: &c2pb.TaskContext{TaskId: int64(existingTasks[0].ID), Jwt: token},
						Output: &c2pb.TaskOutput{
							Id:     int64(existingTasks[0].ID),
							Output: "",
							Error: &c2pb.TaskError{
								Msg: "hello error!",
							},
							ExecStartedAt: now,
						},
					},
				},
			},
			wantResp:   &c2pb.ReportOutputResponse{},
			wantCode:   codes.OK,
			wantOutput: "TestOutput", // Output is additive, previous test ran first? No, tests are independent runs unless I chain them?
			// Tests run in loop. `existingTasks[0]` is modified by previous test case?
			// `t.Run` runs sequentially. The graph state persists across subtests because `graph` is created once in `c2test.New(t)`.
			// Wait, `c2test.New(t)` is called inside `TestReportTaskOutput`.
			// So `graph` is shared across all `t.Run`.
			// So `First_Output` modifies `existingTasks[0]`.
			// `First_error` uses `existingTasks[0]`.
			// So output will be appended.
			// But here output is empty string.
			wantError:         "hello error!",
			wantExecStartedAt: now,
			targetTaskID:      int64(existingTasks[0].ID),
		},
		{
			name: "Append_Output",
			req: &c2pb.ReportOutputRequest{
				Message: &c2pb.ReportOutputRequest_TaskOutput{
					TaskOutput: &c2pb.ReportTaskOutputMessage{
						Context: &c2pb.TaskContext{TaskId: int64(existingTasks[0].ID), Jwt: token},
						Output: &c2pb.TaskOutput{
							Id:     int64(existingTasks[0].ID),
							Output: "_AppendedOutput",
							Error: &c2pb.TaskError{
								Msg: "_AppendEror",
							},
						},
					},
				},
			},
			wantResp:          &c2pb.ReportOutputResponse{},
			wantCode:          codes.OK,
			wantOutput:        "TestOutput_AppendedOutput",
			wantError:         "hello error!_AppendEror",
			wantExecStartedAt: now,
			targetTaskID:      int64(existingTasks[0].ID),
		},
		{
			name: "Exec_Finished",
			req: &c2pb.ReportOutputRequest{
				Message: &c2pb.ReportOutputRequest_TaskOutput{
					TaskOutput: &c2pb.ReportTaskOutputMessage{
						Context: &c2pb.TaskContext{TaskId: int64(existingTasks[0].ID), Jwt: token},
						Output: &c2pb.TaskOutput{
							Id:             int64(existingTasks[0].ID),
							ExecFinishedAt: finishedAt,
						},
					},
				},
			},
			wantResp:           &c2pb.ReportOutputResponse{},
			wantCode:           codes.OK,
			wantOutput:         "TestOutput_AppendedOutput",
			wantError:          "hello error!_AppendEror",
			wantExecStartedAt:  now,
			wantExecFinishedAt: finishedAt,
			targetTaskID:       int64(existingTasks[0].ID),
		},
		{
			name: "Not_Found",
			req: &c2pb.ReportOutputRequest{
				Message: &c2pb.ReportOutputRequest_TaskOutput{
					TaskOutput: &c2pb.ReportTaskOutputMessage{
						Context: &c2pb.TaskContext{TaskId: 999888777666, Jwt: token},
						Output: &c2pb.TaskOutput{
							Id: 999888777666,
						},
					},
				},
			},
			wantResp: nil,
			wantCode: codes.NotFound,
		},
		{
			name: "Invalid_Argument",
			req: &c2pb.ReportOutputRequest{
				Message: &c2pb.ReportOutputRequest_TaskOutput{
					TaskOutput: &c2pb.ReportTaskOutputMessage{
						// Missing context or output
						Output: &c2pb.TaskOutput{},
					},
				},
			},
			wantResp: nil,
			wantCode: codes.InvalidArgument,
		},
	}

	// Run Tests
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			// Callback
			// Set JWT if needed (already set in cases above)
			// But if we wanted to enforce it here:
			if msg, ok := tc.req.Message.(*c2pb.ReportOutputRequest_TaskOutput); ok {
				if msg.TaskOutput.Context == nil {
					// Create it if missing?
					// msg.TaskOutput.Context = &c2pb.TaskContext{Jwt: token}
				} else {
					// msg.TaskOutput.Context.Jwt = token
				}
			}

			resp, err := client.ReportOutput(ctx, tc.req)

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

			// Load Task
			if tc.targetTaskID != 0 {
				testTask, err := graph.Task.Get(ctx, int(tc.targetTaskID))
				require.NoError(t, err)

				// Task Assertions
				assert.Equal(t, tc.wantOutput, testTask.Output)
				assert.Equal(t, tc.wantError, testTask.Error)
			}
		})
	}

}
