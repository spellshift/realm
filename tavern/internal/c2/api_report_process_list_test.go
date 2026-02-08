package c2_test

import (
	"context"
	"testing"

	"github.com/google/go-cmp/cmp"
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

func TestReportProcessList(t *testing.T) {
	// Setup Dependencies
	ctx := context.Background()
	client, graph, close, token := c2test.New(t)
	defer close()

	// Test Data
	existingBeacon := c2test.NewRandomBeacon(ctx, graph)
	existingTask := c2test.NewRandomAssignedTask(ctx, graph, existingBeacon.Identifier)
	existingHost := existingBeacon.QueryHost().OnlyX(ctx)

	// Test Cases
	tests := []struct {
		name string
		host *ent.Host
		task *ent.Task
		req  *c2pb.ReportProcessListRequest

		wantResp     *c2pb.ReportProcessListResponse
		wantCode     codes.Code
		wantHostPIDs []uint64
		wantTaskPIDs []uint64
	}{
		{
			name: "New_List",
			host: existingHost,
			task: existingTask,
			req: &c2pb.ReportProcessListRequest{
				Context: &c2pb.TaskContext{TaskId: int64(existingTask.ID), Jwt: token},
				List: &epb.ProcessList{
					List: []*epb.Process{
						{Pid: 1, Name: "systemd", Principal: "root", Status: epb.Process_STATUS_RUN},
						{Pid: 2321, Name: "/bin/sh", Principal: "root", Status: epb.Process_STATUS_SLEEP},
						{Pid: 4505, Name: "/usr/bin/sshd", Principal: "root", Status: epb.Process_STATUS_RUN},
					},
				},
			},
			wantResp:     &c2pb.ReportProcessListResponse{},
			wantCode:     codes.OK,
			wantHostPIDs: []uint64{1, 2321, 4505},
			wantTaskPIDs: []uint64{1, 2321, 4505},
		},
		{
			name: "Updated_List",
			host: existingHost,
			task: existingTask,
			req: &c2pb.ReportProcessListRequest{
				Context: &c2pb.TaskContext{TaskId: int64(existingTask.ID), Jwt: token},
				List: &epb.ProcessList{
					List: []*epb.Process{
						{Pid: 1, Name: "systemd", Principal: "root"},
						{Pid: 4505, Name: "/usr/bin/sshd", Principal: "root"},
						{Pid: 4809, Name: "/usr/bin/nginx", Principal: "root"},
					},
				},
			},
			wantResp:     &c2pb.ReportProcessListResponse{},
			wantCode:     codes.OK,
			wantHostPIDs: []uint64{1, 4505, 4809},
			wantTaskPIDs: []uint64{1, 2321, 4505, 1, 4505, 4809},
		},
		{
			name: "No_TaskID",
			host: existingHost,
			task: existingTask,
			req: &c2pb.ReportProcessListRequest{
				List: &epb.ProcessList{
					List: []*epb.Process{
						{Pid: 1, Name: "systemd", Principal: "root"},
					},
				},
			},
			wantResp: nil,
			wantCode: codes.InvalidArgument,
		},
		{
			name: "No_Processes",
			host: existingHost,
			task: existingTask,
			req: &c2pb.ReportProcessListRequest{
				Context: &c2pb.TaskContext{TaskId: int64(existingTask.ID), Jwt: token},
				List: &epb.ProcessList{
					List: []*epb.Process{},
				},
			},
			wantResp: nil,
			wantCode: codes.InvalidArgument,
		},
		{
			name: "Not_Found",
			req: &c2pb.ReportProcessListRequest{
				Context: &c2pb.TaskContext{TaskId: 99888777776666, Jwt: token},
				List: &epb.ProcessList{
					List: []*epb.Process{
						{Pid: 1, Name: "systemd", Principal: "root"},
					},
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
			resp, err := client.ReportProcessList(ctx, tc.req)

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

			// Assert Task Processes
			var taskPIDs []uint64
			taskProcessList := tc.task.QueryReportedProcesses().AllX(ctx)
			for _, proc := range taskProcessList {
				taskPIDs = append(taskPIDs, proc.Pid)
			}
			assert.ElementsMatch(t, tc.wantTaskPIDs, taskPIDs)

			// Assert Host Processes
			var hostPIDs []uint64
			hostProcessList := tc.host.QueryProcesses().AllX(ctx)
			for _, proc := range hostProcessList {
				hostPIDs = append(hostPIDs, proc.Pid)
			}
			assert.ElementsMatch(t, tc.wantHostPIDs, hostPIDs)
		})
	}
}
