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
	ctx := stream.Context()

	var (
		path        string
		owner       string
		group       string
		permissions string
		size        uint64
		hash        string
		kind        c2pb.ReportFileKind

		host      *ent.Host
		task      *ent.Task
		shellTask *ent.ShellTask

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

		// The initial message must carry the context so the stream can be
		// validated and authorized before any chunk data is buffered.
		if host == nil {
			if tc := req.GetTaskContext(); tc != nil {
				beaconID, err := srv.ValidateJWT(tc.Jwt)
				if err != nil {
					return err
				}
				t, err := srv.loadTaskForBeacon(ctx, int(tc.TaskId), beaconID)
				if err != nil {
					return err
				}
				task = t
				h, err := t.QueryBeacon().QueryHost().Only(ctx)
				if err != nil {
					return status.Errorf(codes.Internal, "failed to load host from task: %v", err)
				}
				host = h
			} else if stc := req.GetShellTaskContext(); stc != nil {
				beaconID, err := srv.ValidateJWT(stc.Jwt)
				if err != nil {
					return err
				}
				st, err := srv.loadShellTaskForBeacon(ctx, int(stc.ShellTaskId), beaconID)
				if err != nil {
					return err
				}
				shellTask = st
				h, err := st.QueryShell().QueryBeacon().QueryHost().Only(ctx)
				if err != nil {
					return status.Errorf(codes.Internal, "failed to load host from shell task: %v", err)
				}
				host = h
			} else {
				return status.Errorf(codes.InvalidArgument, "missing task context on initial message")
			}
		}

		// Collect args
		if kind == c2pb.ReportFileKind_REPORT_FILE_KIND_UNSPECIFIED {
			kind = req.GetKind()
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
	if path == "" {
		return status.Errorf(codes.InvalidArgument, "must provide valid path")
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

	if isScreenshot {
		builder := client.Screenshot.Create().
			SetHostID(host.ID).
			SetName(path).
			SetSize(size).
			SetHash(hash).
			SetContent(content)

		if task != nil {
			builder.SetTaskID(task.ID)
		}
		if shellTask != nil {
			builder.SetShellTaskID(shellTask.ID)
		}

		_, err = builder.Save(ctx)
		if err != nil {
			return rollback(tx, fmt.Errorf("failed to create screenshot: %w", err))
		}
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

	return stream.SendAndClose(&c2pb.ReportFileResponse{})
}
