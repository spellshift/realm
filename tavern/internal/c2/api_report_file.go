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
		if req.Chunk == nil {
			continue
		}
		if taskID == 0 && shellTaskID == 0 {
			switch c := req.Context.(type) {
			case *c2pb.ReportFileRequest_TaskContext:
				taskID = c.TaskContext.TaskId
				jwtToken = c.TaskContext.Jwt
			case *c2pb.ReportFileRequest_ShellContext:
				shellTaskID = c.ShellContext.TaskId
				jwtToken = c.ShellContext.Jwt
			}
		}
		// Fallback if context was not provided in the first message or correctly parsed above
		if jwtToken == "" {
			switch c := req.Context.(type) {
			case *c2pb.ReportFileRequest_TaskContext:
				jwtToken = c.TaskContext.Jwt
			case *c2pb.ReportFileRequest_ShellContext:
				jwtToken = c.ShellContext.Jwt
			}
		}

		if path == "" && req.Chunk.Metadata != nil {
			path = req.Chunk.Metadata.GetPath()
		}
		if owner == "" && req.Chunk.Metadata != nil {
			owner = req.Chunk.Metadata.GetOwner()
		}
		if group == "" && req.Chunk.Metadata != nil {
			group = req.Chunk.Metadata.GetGroup()
		}
		if permissions == "" && req.Chunk.Metadata != nil {
			permissions = req.Chunk.Metadata.GetPermissions()
		}
		if size == 0 && req.Chunk.Metadata != nil {
			size = req.Chunk.Metadata.GetSize()
		}
		if hash == "" && req.Chunk.Metadata != nil {
			hash = req.Chunk.Metadata.GetSha3_256Hash()
		}
		content = append(content, req.Chunk.GetChunk()...)
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
	var shell *ent.Shell

	if taskID != 0 {
		// Load Task
		task, err = srv.graph.Task.Get(stream.Context(), int(taskID))
		if ent.IsNotFound(err) {
			return status.Errorf(codes.NotFound, "failed to find related task")
		}
		if err != nil {
			return status.Errorf(codes.Internal, "failed to load task: %v", err)
		}

		// Load Host
		host, err = task.QueryBeacon().QueryHost().Only(stream.Context())
		if err != nil {
			return status.Errorf(codes.Internal, "failed to load host")
		}
	} else {
		// Load ShellTask
		shellTask, err = srv.graph.ShellTask.Get(stream.Context(), int(shellTaskID))
		if ent.IsNotFound(err) {
			return status.Errorf(codes.NotFound, "failed to find related shell task")
		}
		if err != nil {
			return status.Errorf(codes.Internal, "failed to load shell task: %v", err)
		}

		// Load Shell
		shell, err = shellTask.QueryShell().Only(stream.Context())
		if err != nil {
			return status.Errorf(codes.Internal, "failed to load shell")
		}

		// Load Host
		host, err = shell.QueryBeacon().QueryHost().Only(stream.Context())
		if err != nil {
			return status.Errorf(codes.Internal, "failed to load host")
		}
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
	builder := client.HostFile.Create().
		SetHostID(host.ID).
		SetPath(path).
		SetOwner(owner).
		SetGroup(group).
		SetPermissions(permissions).
		SetSize(size).
		SetHash(hash).
		SetContent(content)

	if task != nil {
		builder.SetTask(task)
	}
	if shell != nil {
		builder.SetShell(shell)
	}
	if shellTask != nil {
		builder.SetShellTask(shellTask)
	}

	f, err := builder.Save(stream.Context())
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
