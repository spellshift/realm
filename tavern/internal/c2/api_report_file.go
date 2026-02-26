package c2

import (
	"fmt"
	"io"
	"unicode/utf8"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/hostfile"
)

func (srv *Server) ReportFile(stream c2pb.C2_ReportFileServer) error {

	var (
		taskID      int64
		shellTaskID int64
		jwtToken    string
		path        string
		owner       string
		group       string
		permissions string
		size        uint64
		hash        string

		content []byte
	)

	ctx := stream.Context()

	// Loop Input Stream
	for {
		req, err := stream.Recv()
		if err == io.EOF {
			break
		}
		if err != nil {
			return status.Errorf(codes.Internal, "failed to receive report_file request: %v", err)
		}

		// Collect args
		if taskID == 0 && shellTaskID == 0 {
			if tc := req.GetTaskContext(); tc != nil {
				taskID = tc.TaskId
				jwtToken = tc.Jwt
			} else if stc := req.GetShellTaskContext(); stc != nil {
				shellTaskID = stc.ShellTaskId
				jwtToken = stc.Jwt
			}
		}

		if req.Chunk != nil {
			if path == "" && req.Chunk.Metadata != nil {
				path = req.Chunk.Metadata.GetPath()
				owner = req.Chunk.Metadata.GetOwner()
				group = req.Chunk.Metadata.GetGroup()
				permissions = req.Chunk.Metadata.GetPermissions()
				size = req.Chunk.Metadata.GetSize()
				hash = req.Chunk.Metadata.GetSha3_256Hash()
			}
			content = append(content, req.Chunk.GetChunk()...)
		}
	}

	// Input Validation
	if taskID == 0 && shellTaskID == 0 {
		return status.Errorf(codes.InvalidArgument, "must provide valid task id or shell task id")
	}
	if path == "" {
		return status.Errorf(codes.InvalidArgument, "must provide valid path")
	}

	err := srv.ValidateJWT(jwtToken)
	if err != nil {
		return err
	}

	var host *ent.Host
	var task *ent.Task
	var shellTask *ent.ShellTask

	if taskID != 0 {
		t, err := srv.graph.Task.Get(ctx, int(taskID))
		if ent.IsNotFound(err) {
			return status.Errorf(codes.NotFound, "failed to find related task")
		}
		if err != nil {
			return status.Errorf(codes.Internal, "failed to load task: %v", err)
		}
		task = t
		h, err := t.QueryBeacon().QueryHost().Only(ctx)
		if err != nil {
			return status.Errorf(codes.Internal, "failed to load host from task: %v", err)
		}
		host = h
	} else {
		st, err := srv.graph.ShellTask.Get(ctx, int(shellTaskID))
		if ent.IsNotFound(err) {
			return status.Errorf(codes.NotFound, "failed to find related shell task")
		}
		if err != nil {
			return status.Errorf(codes.Internal, "failed to load shell task: %v", err)
		}
		shellTask = st
		h, err := st.QueryShell().QueryBeacon().QueryHost().Only(ctx)
		if err != nil {
			return status.Errorf(codes.Internal, "failed to load host from shell task: %v", err)
		}
		host = h
	}

	// Load Existing Files
	existingFiles, err := host.QueryFiles().
		Where(
			hostfile.Path(path),
		).All(ctx)
	if err != nil {
		return status.Errorf(codes.Internal, "failed to load existing host files: %v", err)
	}

	// Prepare Transaction
	tx, err := srv.graph.Tx(ctx)
	if err != nil {
		return status.Errorf(codes.Internal, "failed to initialize transaction: %v", err)
	}
	client := tx.Client()

	// Rollback transaction if we panic
	defer func() {
		if v := recover(); v != nil {
			tx.Rollback()
			panic(v)
		}
	}()

	// Create File
	builder := client.HostFile.Create().
		SetHostID(host.ID).
		SetPath(path).
		SetOwner(owner).
		SetGroup(group).
		SetPermissions(permissions).
		SetSize(size).
		SetHash(hash).
		SetContent(content)

	// Derive Preview
	const maxPreviewSize = 100 * 1024
	if len(content) > 0 && utf8.Valid(content){
		builder.SetPreviewType(hostfile.PreviewTypeTEXT)
		builder.SetPreview(content[:min(len(content), maxPreviewSize)])
	} else {
		builder.SetPreviewType(hostfile.PreviewTypeNONE)
	}

	if task != nil {
		builder.SetTaskID(task.ID)
	}
	if shellTask != nil {
		builder.SetShellTaskID(shellTask.ID)
	}

	f, err := builder.Save(ctx)
	if err != nil {
		return rollback(tx, fmt.Errorf("failed to create host file: %w", err))
	}

	// Clear Previous Files, Set New File
	_, err = client.Host.UpdateOneID(host.ID).
		AddFiles(f).
		RemoveFiles(existingFiles...).
		Save(ctx)
	if err != nil {
		return rollback(tx, fmt.Errorf("failed to remove previous host files: %w", err))
	}

	// Commit Transaction
	if err := tx.Commit(); err != nil {
		return rollback(tx, fmt.Errorf("failed to commit transaction: %w", err))
	}

	return stream.SendAndClose(&c2pb.ReportFileResponse{})
}
