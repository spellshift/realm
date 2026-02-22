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

func TestReportFile(t *testing.T) {
	// Setup Dependencies
	ctx := context.Background()
	client, graph, close, token := c2test.New(t)
	defer close()

	// Test Data
	existingBeacons := []*ent.Beacon{
		c2test.NewRandomBeacon(ctx, graph),
		c2test.NewRandomBeacon(ctx, graph),
	}
	existingHosts := []*ent.Host{
		existingBeacons[0].QueryHost().OnlyX(ctx),
		existingBeacons[1].QueryHost().OnlyX(ctx),
	}
	existingTasks := []*ent.Task{
		c2test.NewRandomAssignedTask(ctx, graph, existingBeacons[0].Identifier),
		c2test.NewRandomAssignedTask(ctx, graph, existingBeacons[0].Identifier),
		c2test.NewRandomAssignedTask(ctx, graph, existingBeacons[0].Identifier),
		c2test.NewRandomAssignedTask(ctx, graph, existingBeacons[1].Identifier),
	}
	existingHostFiles := []*ent.HostFile{
		graph.HostFile.Create().
			SetPath("/existing/path").
			SetOwner("test_user").
			SetHost(existingHosts[0]).
			SetTask(existingTasks[0]).
			SaveX(ctx),
		graph.HostFile.Create().
			SetPath("/existing/path_2").
			SetOwner("test_user").
			SetHost(existingHosts[0]).
			SetTask(existingTasks[1]).
			SaveX(ctx),
		graph.HostFile.Create().
			SetPath("/existing/path").
			SetOwner("test_user").
			SetHost(existingHosts[0]).
			SetTask(existingTasks[1]).
			SaveX(ctx),
	}
	existingHosts[0].Update().
		AddFiles(
			existingHostFiles[1],
			existingHostFiles[2],
		).
		SaveX(ctx)

	// Test Cases
	tests := []struct {
		name string
		reqs []*c2pb.ReportFileRequest
		host *ent.Host

		wantCode        codes.Code
		wantResp        *c2pb.ReportFileResponse
		wantHostFiles   []string
		wantPath        string
		wantOwner       string
		wantGroup       string
		wantPermissions string
		wantSize        uint64
		wantHash        string
		wantContent     []byte
	}{
		{
			name: "MissingTaskID",
			reqs: []*c2pb.ReportFileRequest{
				{
					Chunk: &epb.File{
						Metadata: &epb.FileMetadata{
							Path: "/test",
						},
					},
				},
			},
			wantCode: codes.InvalidArgument,
		},
		{
			name: "MissingPath",
			reqs: []*c2pb.ReportFileRequest{
				{
					Context: &c2pb.ReportFileRequest_TaskContext{
						TaskContext: &c2pb.TaskContext{TaskId: 1234, Jwt: token},
					},
				},
			},
			wantCode: codes.InvalidArgument,
		},
		{
			name: "NewFile_Single",
			reqs: []*c2pb.ReportFileRequest{
				{
					Context: &c2pb.ReportFileRequest_TaskContext{
						TaskContext: &c2pb.TaskContext{TaskId: int64(existingTasks[2].ID), Jwt: token},
					},
					Chunk: &epb.File{
						Metadata: &epb.FileMetadata{
							Path:         "/new/file",
							Owner:        "root",
							Group:        "wheel",
							Permissions:  "0664",
							Size:         999999,
							Sha3_256Hash: "I_AM_IGNORED",
						},
						Chunk: []byte("death"),
					},
				},
			},
			host:     existingHosts[0],
			wantCode: codes.OK,
			wantResp: &c2pb.ReportFileResponse{},
			wantHostFiles: []string{
				"/existing/path",
				"/existing/path_2",
				"/new/file",
			},
			wantPath:        "/new/file",
			wantOwner:       "root",
			wantGroup:       "wheel",
			wantPermissions: "0664",
			wantSize:        5,
			wantHash:        "da4b6723781fc3c92cf4e303532668f1352034a4250efa47f225a4243e33c89b",
			wantContent:     []byte("death"),
		},
		{
			name: "NewFile_MultiChunk",
			reqs: []*c2pb.ReportFileRequest{
				{
					Context: &c2pb.ReportFileRequest_TaskContext{
						TaskContext: &c2pb.TaskContext{TaskId: int64(existingTasks[2].ID), Jwt: token},
					},
					Chunk: &epb.File{
						Metadata: &epb.FileMetadata{
							Path: "/another/new/file",
						},
						Chunk: []byte("death"),
					},
				},
				{
					Chunk: &epb.File{
						Chunk: []byte("note"),
					},
				},
			},
			host:     existingHosts[0],
			wantCode: codes.OK,
			wantResp: &c2pb.ReportFileResponse{},
			wantHostFiles: []string{
				"/existing/path_2",
				"/existing/path",
				"/new/file",
				"/another/new/file",
			},
			wantPath:    "/another/new/file",
			wantSize:    9,
			wantHash:    "a89332a42f5fbfcda0711dd7615aee897a9977f2b6adf12bb2db41a1b9f79a90",
			wantContent: []byte("deathnote"),
		},
		{
			name: "Replace_File",
			reqs: []*c2pb.ReportFileRequest{
				{
					Context: &c2pb.ReportFileRequest_TaskContext{
						TaskContext: &c2pb.TaskContext{TaskId: int64(existingTasks[2].ID), Jwt: token},
					},
					Chunk: &epb.File{
						Metadata: &epb.FileMetadata{
							Path: "/another/new/file",
						},
						Chunk: []byte("replaced"),
					},
				},
			},
			host:     existingHosts[0],
			wantCode: codes.OK,
			wantResp: &c2pb.ReportFileResponse{},
			wantHostFiles: []string{
				"/existing/path_2",
				"/existing/path",
				"/new/file",
				"/another/new/file",
			},
			wantPath:    "/another/new/file",
			wantSize:    8,
			wantHash:    "e0f00440c4d0ee2fd0b63b59402faf9a9d6b6c26a41c2353141328ae8df80832",
			wantContent: []byte("replaced"),
		},
		{
			name: "No_Prexisting_Files",
			reqs: []*c2pb.ReportFileRequest{
				{
					Context: &c2pb.ReportFileRequest_TaskContext{
						TaskContext: &c2pb.TaskContext{TaskId: int64(existingTasks[3].ID), Jwt: token},
					},
					Chunk: &epb.File{
						Metadata: &epb.FileMetadata{
							Path: "/no/other/files",
						},
						Chunk: []byte("meow"),
					},
				},
			},
			host:          existingHosts[1],
			wantCode:      codes.OK,
			wantResp:      &c2pb.ReportFileResponse{},
			wantHostFiles: []string{"/no/other/files"},
			wantPath:      "/no/other/files",
			wantSize:      4,
			wantHash:      "ecb287a944d62ba58b7e7310529172a9c121957c2edea47a948919c342ca9467",
			wantContent:   []byte("meow"),
		},
	}

	// Run Tests
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			// gRPC
			rClient, err := client.ReportFile(ctx)
			require.NoError(t, err)
			for _, req := range tc.reqs {
				rClient.Send(req)
			}
			resp, err := rClient.CloseAndRecv()

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

			// Load Files
			testHost := graph.Host.GetX(ctx, tc.host.ID)
			testHostFiles := testHost.QueryFiles().AllX(ctx)
			testHostFilePaths := make([]string, 0, len(testHostFiles))
			var testFile *ent.HostFile
			for _, f := range testHostFiles {
				testHostFilePaths = append(testHostFilePaths, f.Path)
				if f.Path == tc.wantPath {
					testFile = f
				}
			}
			require.NotNil(t, testFile, "%q file was not associated with host", tc.wantPath)

			// Assert Files
			sorter := func(a, b string) bool { return a < b }
			if diff := cmp.Diff(tc.wantHostFiles, testHostFilePaths, cmpopts.SortSlices(sorter)); diff != "" {
				t.Errorf("invalid host file associations (-want +got): %v", diff)
			}
			assert.Equal(t, tc.wantPath, testFile.Path)
			assert.Equal(t, tc.wantOwner, testFile.Owner)
			assert.Equal(t, tc.wantGroup, testFile.Group)
			assert.Equal(t, tc.wantPermissions, testFile.Permissions)
			assert.Equal(t, tc.wantSize, testFile.Size)
			assert.Equal(t, tc.wantHash, testFile.Hash)
		})
	}

}
