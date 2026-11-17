package graphql

import (
	"context"
	"sort"
	"strconv"

	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/beacon"
	"realm.pub/tavern/internal/logging"
)

var (
	mutationLog = logging.For("graphql.mutation")
	queryLog    = logging.For("graphql.query")
)

// auditCtx ensures the context carries graphql entrypoint metadata, trace/request IDs,
// and Actor attribution derived from auth.UserFromContext when not already populated.
func auditCtx(ctx context.Context) context.Context {
	if ctx == nil {
		ctx = context.Background()
	}
	if logging.EntrypointFromContext(ctx) == "" {
		ctx = logging.WithEntrypoint(ctx, "graphql")
	}
	if logging.ActorFromContext(ctx).ID == "" {
		if u := auth.UserFromContext(ctx); u != nil {
			ctx = logging.WithActor(ctx, logging.Actor{
				Kind:    logging.ActorUser,
				ID:      strconv.Itoa(u.ID),
				Name:    u.Name,
				IsAdmin: u.IsAdmin,
			})
		}
	}
	ctx, _, _ = logging.EnsureTrace(ctx)
	return ctx
}

func resolveHostIDsForBeacons(ctx context.Context, client *ent.Client, beaconIDs []int) []int {
	if client == nil || len(beaconIDs) == 0 {
		return nil
	}
	beacons, err := client.Beacon.Query().
		Where(beacon.IDIn(beaconIDs...)).
		WithHost().
		All(ctx)
	if err != nil {
		mutationLog.WarnContext(ctx, "failed to resolve host IDs for quest audit", "error", err)
		return nil
	}
	seen := make(map[int]struct{}, len(beacons))
	hostIDs := make([]int, 0, len(beacons))
	for _, b := range beacons {
		if b.Edges.Host == nil {
			continue
		}
		hid := b.Edges.Host.ID
		if _, ok := seen[hid]; ok {
			continue
		}
		seen[hid] = struct{}{}
		hostIDs = append(hostIDs, hid)
	}
	sort.Ints(hostIDs)
	return hostIDs
}

func collectTagIDs(tags []*ent.Tag) []int {
	ids := make([]int, 0, len(tags))
	for _, t := range tags {
		if t != nil {
			ids = append(ids, t.ID)
		}
	}
	return ids
}

func linkAuditAttrs(ctx context.Context, l *ent.Link, fallbackAssetID int) []any {
	if l == nil {
		return nil
	}
	assetID := fallbackAssetID
	if assetID == 0 {
		if id, err := l.QueryAsset().OnlyID(ctx); err == nil {
			assetID = id
		}
	}
	var downloadLimit any
	if l.DownloadLimit != nil {
		downloadLimit = *l.DownloadLimit
	}
	attrs := []any{
		"link_id", l.ID,
		"link_path", l.Path,
		"download_limit", downloadLimit,
	}
	if assetID > 0 {
		attrs = append(attrs, "asset_id", assetID)
	}
	return attrs
}
