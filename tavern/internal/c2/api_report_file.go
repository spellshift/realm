package c2

import (
	"fmt"
	"io"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/hostfile"
)

func (srv *Server) ReportFile(stream c2pb.C2_ReportFileServer) error {

	var (
		taskID      int64
		path        string
		owner       string
		group       string
		permissions string
		size        int
		hash        string

		content []byte
	)

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
		if taskID == 0 {
			taskID = req.GetTaskId()
		}
		if path == "" {
			path = req.GetPath()
		}
		if owner == "" {
			owner = req.GetOwner()
		}
		if group == "" {
			group = req.GetGroup()
		}
		if permissions == "" {
			permissions = req.GetPermissions()
		}
		if size == 0 {
			size = int(req.GetSize())
		}
		if hash == "" {
			hash = req.GetSha3_256Hash()
		}
		content = append(content, req.GetChunk()...)
	}

	// Input Validation
	if taskID == 0 {
		return status.Errorf(codes.InvalidArgument, "must provide valid task id")
	}
	if path == "" {
		return status.Errorf(codes.InvalidArgument, "must provide valid path")
	}

	// Load Task
	task, err := srv.graph.Task.Get(stream.Context(), int(taskID))
	if ent.IsNotFound(err) {
		return status.Errorf(codes.NotFound, "failed to find related task")
	}
	if err != nil {
		return status.Errorf(codes.Internal, "failed to load task: %v", err)
	}

	// Load Host
	host, err := task.QueryBeacon().QueryHost().Only(stream.Context())
	if err != nil {
		return status.Errorf(codes.Internal, "failed to load host")
	}

	// Load Existing Files
	existingFiles, err := host.QueryFiles().
		Where(
			hostfile.Path(path),
		).All(stream.Context())
	if err != nil {
		return status.Errorf(codes.Internal, "failed to load existing host files: %v", err)
	}

	// Prepare Transaction
	tx, err := srv.graph.Tx(stream.Context())
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
	f, err := client.HostFile.Create().
		SetHostID(host.ID).
		SetTaskID(task.ID).
		SetPath(path).
		SetOwner(owner).
		SetGroup(group).
		SetPermissions(permissions).
		SetSize(size).
		SetHash(hash).
		SetContent(content).
		Save(stream.Context())
	if err != nil {
		return rollback(tx, fmt.Errorf("failed to create host file: %w", err))
	}

	// Clear Previous Files, Set New File
	_, err = client.Host.UpdateOneID(host.ID).
		AddFiles(f).
		RemoveFiles(existingFiles...).
		Save(stream.Context())
	if err != nil {
		return rollback(tx, fmt.Errorf("failed to remove previous host files: %w", err))
	}

	// Commit Transaction
	if err := tx.Commit(); err != nil {
		return rollback(tx, fmt.Errorf("failed to commit transaction: %w", err))
	}

	return stream.SendAndClose(&c2pb.ReportFileResponse{})
}
