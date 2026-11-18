package c2

import (
	"bytes"
	"errors"
	"fmt"

	"golang.org/x/crypto/sha3"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/asset"
	"realm.pub/tavern/internal/logging"
	"realm.pub/tavern/internal/tomebundle"
)

func (srv *Server) FetchAsset(req *c2pb.FetchAssetRequest, stream c2pb.C2_FetchAssetServer) error {
	ctx, _ := enrichC2Context(stream.Context())

	var jwt string
	if tc := req.GetTaskContext(); tc != nil {
		jwt = tc.GetJwt()
	} else if stc := req.GetShellTaskContext(); stc != nil {
		jwt = stc.GetJwt()
	} else {
		return status.Errorf(codes.InvalidArgument, "missing context")
	}

	ctx, beaconID, err := srv.ValidateJWTContext(ctx, jwt)
	if err != nil {
		return err
	}

	name := req.GetName()
	var content []byte
	var checksum string
	var assetID int
	var bundleID int
	if tc := req.GetTaskContext(); tc != nil && tc.GetTaskId() != 0 {
		t, err := srv.loadTaskForBeacon(ctx, int(tc.GetTaskId()), beaconID)
		if ent.IsNotFound(err) {
			return status.Error(codes.NotFound, "task not found for beacon")
		}
		if err != nil {
			return status.Errorf(codes.Internal, "failed to load task: %v", err)
		}
		q, err := t.QueryQuest().Only(ctx)
		if err != nil {
			return status.Errorf(codes.Internal, "failed to load quest: %v", err)
		}
		isScheduled, err := q.QueryScheduledTask().Exist(ctx)
		if err != nil {
			return status.Errorf(codes.Internal, "failed to identify scheduled quest: %v", err)
		}
		if isScheduled {
			bundle, err := q.QueryBundle().Only(ctx)
			if err != nil && !ent.IsNotFound(err) {
				return status.Errorf(codes.Internal, "failed to load quest bundle: %v", err)
			}
			if bundle != nil {
				content, err = tomebundle.Read(bundle.Content, name)
				if errors.Is(err, tomebundle.ErrFileNotFound) {
					return status.Errorf(codes.NotFound, "asset %q not found in quest bundle", name)
				}
				if err != nil {
					return status.Errorf(codes.Internal, "failed to read quest bundle: %v", err)
				}
				bundleID = bundle.ID
				checksum = fmt.Sprintf("%x", sha3.Sum256(content))
			}
		}
	}
	if bundleID == 0 {
		// Shell tasks, manual quests, and legacy scheduled quests use the
		// existing asset lookup behavior.
		a, err := srv.graph.Asset.Query().Where(asset.Name(name)).Only(ctx)
		if ent.IsNotFound(err) {
			return status.Errorf(codes.NotFound, "%v", err)
		}
		if err != nil {
			return status.Errorf(codes.Internal, "failed to query asset (%q): %v", name, err)
		}
		content, checksum, assetID = a.Content, a.Hash, a.ID
	}

	// Set Header Metadata
	stream.SetHeader(metadata.Pairs(
		"sha3-256-checksum", checksum,
		"file-size", fmt.Sprintf("%d", len(content)),
	))

	// Send Asset Chunks
	buf := bytes.NewBuffer(content)
	for {
		// Check Empty Buffer
		if buf.Len() < 1 {
			auditAttrs := make([]any, 0, 10)
			if bundleID != 0 {
				auditAttrs = append(auditAttrs, "bundle_id", bundleID, "asset_name", name, "asset_hash", checksum, "asset_size", len(content))
			} else {
				auditAttrs = append(auditAttrs, logging.AssetAttrs(assetID, name, checksum, len(content))...)
			}
			auditAttrs = append(auditAttrs, "beacon_id", beaconID)
			logging.Audit(ctx, "c2", "c2.asset_fetched", "beacon fetched asset", auditAttrs...)
			return nil
		}

		// Determine Chunk Size
		chunkLen := srv.MaxFileChunkSize
		if uint64(buf.Len()) < chunkLen {
			chunkLen = uint64(buf.Len())
		}

		// Read Chunk
		chunk := make([]byte, chunkLen)
		if _, err := buf.Read(chunk); err != nil {
			return status.Errorf(codes.Internal, "failed to read file content: %v", err)
		}

		// Send Chunk
		sendErr := stream.Send(&c2pb.FetchAssetResponse{
			Chunk: chunk,
		})
		if sendErr != nil {
			return status.Errorf(codes.Internal, "failed to send file content: %v", sendErr)
		}
		reportLog.TraceContext(ctx, "sent asset chunk", "asset_id", assetID, "bundle_id", bundleID, "chunk_size", len(chunk))
	}
}
