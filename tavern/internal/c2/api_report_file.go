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
	"realm.pub/tavern/internal/logging"
)

var (
	// MaxReportFileSize is the maximum total payload size (in bytes) accepted by ReportFile
	// for non-screenshot host files, preventing unbounded memory buffering.
	MaxReportFileSize = 100 * 1024 * 1024 // 100 MiB

	// MaxScreenshotSize is the maximum total payload size (in bytes) accepted by ReportFile
	// for screenshots, preventing unbounded memory buffering.
	MaxScreenshotSize = 50 * 1024 * 1024 // 50 MiB
)

func (srv *Server) ReportFile(stream c2pb.C2_ReportFileServer) error {
	var (
		taskID        int64
		shellTaskID   int64
		jwtToken      string
		beaconID      int
		authenticated bool
		path          string
		owner         string
		group         string
		permissions   string
		size          uint64
		hash          string
		kind          c2pb.ReportFileKind

		content []byte
	)

	ctx, _ := enrichC2Context(stream.Context())

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
		if kind == c2pb.ReportFileKind_REPORT_FILE_KIND_UNSPECIFIED {
			kind = req.GetKind()
		}

		if !authenticated {
			if tc := req.GetTaskContext(); tc != nil {
				taskID = tc.TaskId
				jwtToken = tc.Jwt
			} else if stc := req.GetShellTaskContext(); stc != nil {
				shellTaskID = stc.ShellTaskId
				jwtToken = stc.Jwt
			}

			if taskID == 0 && shellTaskID == 0 {
				return status.Errorf(codes.InvalidArgument, "must provide valid task id or shell task id")
			}

			ctx, beaconID, err = srv.ValidateJWTContext(ctx, jwtToken)
			if err != nil {
				return err
			}
			authenticated = true
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
			chunk := req.Chunk.GetChunk()
			maxSize := MaxReportFileSize
			if kind == c2pb.ReportFileKind_REPORT_FILE_KIND_SCREENSHOT {
				maxSize = MaxScreenshotSize
			}
			if int64(len(content))+int64(len(chunk)) > int64(maxSize) {
				return status.Errorf(codes.ResourceExhausted, "reported file exceeds maximum allowed size of %d bytes", maxSize)
			}
			content = append(content, chunk...)
		}
	}

	// Input Validation
	if !authenticated {
		return status.Errorf(codes.InvalidArgument, "must provide valid task id or shell task id")
	}
	if path == "" {
		return status.Errorf(codes.InvalidArgument, "must provide valid path")
	}

	var host *ent.Host
	var task *ent.Task
	var shellTask *ent.ShellTask

	if taskID != 0 {
		t, err := srv.loadTaskForBeacon(ctx, int(taskID), beaconID)
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
		st, err := srv.loadShellTaskForBeacon(ctx, int(shellTaskID), beaconID)
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

	isScreenshot := kind == c2pb.ReportFileKind_REPORT_FILE_KIND_SCREENSHOT

	// Load Existing Files (only for HostFile)
	var existingFiles []*ent.HostFile
	if !isScreenshot {
		var err error
		existingFiles, err = host.QueryFiles().
			Where(
				hostfile.Path(path),
			).All(ctx)
		if err != nil {
			return status.Errorf(codes.Internal, "failed to load existing host files: %v", err)
		}
	}

	// Generate a low resolution thumbnail for use in thumbnail list views. This only depends on
	// the already-buffered content (not the DB), so it is done before opening the transaction to
	// avoid holding a DB connection/transaction open for the duration of image processing. If the
	// content cannot be decoded as an image (or exceeds safe decode limits), skip the thumbnail
	// rather than fail the whole report.
	var thumbnail []byte
	if isScreenshot {
		thumbnail, _ = generateScreenshotThumbnail(content)
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

	savedSize := size
	savedHash := hash
	if isScreenshot {
		builder := client.Screenshot.Create().
			SetHostID(host.ID).
			SetName(path).
			SetSize(size).
			SetHash(hash).
			SetContent(content)

		if len(thumbnail) > 0 {
			builder.SetThumbnail(thumbnail)
		}

		if task != nil {
			builder.SetTaskID(task.ID)
		}
		if shellTask != nil {
			builder.SetShellTaskID(shellTask.ID)
		}

		sc, err := builder.Save(ctx)
		if err != nil {
			return rollback(tx, fmt.Errorf("failed to create screenshot: %w", err))
		}
		savedSize = sc.Size
		savedHash = sc.Hash
	} else {
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
		if len(content) > 0 && utf8.Valid(content) {
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
		savedSize = f.Size
		savedHash = f.Hash

		// Clear Previous Files, Set New File
		_, err = client.Host.UpdateOneID(host.ID).
			AddFiles(f).
			RemoveFiles(existingFiles...).
			Save(ctx)
		if err != nil {
			return rollback(tx, fmt.Errorf("failed to remove previous host files: %w", err))
		}
	}

	// Commit Transaction
	if err := tx.Commit(); err != nil {
		return rollback(tx, fmt.Errorf("failed to commit transaction: %w", err))
	}

	auditAttrs := []any{
		"host_id", host.ID,
		"beacon_id", beaconID,
		"path", path,
		"size", savedSize,
		"hash", savedHash,
		"kind", kind.String(),
	}
	if task != nil {
		auditAttrs = append(auditAttrs, "task_id", task.ID)
	}
	if shellTask != nil {
		auditAttrs = append(auditAttrs, "shell_task_id", shellTask.ID)
	}
	logging.Audit(ctx, "c2", "c2.file_reported", "beacon reported file", auditAttrs...)

	return stream.SendAndClose(&c2pb.ReportFileResponse{})
}
