package graphql

import (
	"context"
	"fmt"
	"sort"

	"realm.pub/tavern/internal/autogroup"
	"realm.pub/tavern/internal/autogroup/features"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/quest"
	"realm.pub/tavern/internal/ent/tag"
	"realm.pub/tavern/internal/ent/task"
	"realm.pub/tavern/internal/ent/tome"
	"realm.pub/tavern/internal/namegen"
)

// autogroupTomeName is the tome whose tasks supply the process data host
// autogrouping clusters on.
const autogroupTomeName = "Process list"

// runHostAutogroup clusters every host that has "Process list" data into
// role groups and creates one service-kind Tag per cluster, named with a
// random namegen name and attached to the cluster's hosts.
//
// k, if non-nil, pins the exact number of groups instead of auto-selecting
// it via silhouette score -- the single biggest accuracy lever when the
// caller has a rough idea how many role-types to expect (see
// tavern/internal/autogroup's doc comments). It must be a positive integer
// less than the number of hosts with process-list data.
//
// Tags from previous runs are left untouched. If a generated name collides
// with an existing tag, the whole run is rolled back and an error returned.
func runHostAutogroup(ctx context.Context, client *ent.Client, k *int) ([]*ent.Tag, error) {
	if k != nil && *k <= 0 {
		return nil, fmt.Errorf("k must be a positive integer, got %d", *k)
	}

	tasks, err := client.Task.Query().
		Where(task.HasQuestWith(quest.HasTomeWith(tome.NameEqualFold(autogroupTomeName)))).
		WithBeacon(func(bq *ent.BeaconQuery) { bq.WithHost() }).
		WithReportedProcesses().
		All(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to query %q tasks: %w", autogroupTomeName, err)
	}

	hostIDs, labels, err := clusterHostsByRole(tasks, k)
	if err != nil {
		return nil, err
	}

	// Group host ids by cluster, iterating clusters in a stable order.
	byCluster := map[int][]int{}
	for i, l := range labels {
		byCluster[l] = append(byCluster[l], hostIDs[i])
	}
	clusterIDs := make([]int, 0, len(byCluster))
	for id := range byCluster {
		clusterIDs = append(clusterIDs, id)
	}
	sort.Ints(clusterIDs)

	tx, err := client.Tx(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to initialize transaction: %w", err)
	}
	createdIDs := make([]int, 0, len(clusterIDs))
	for _, id := range clusterIDs {
		name := namegen.NewSimple()
		exists, err := tx.Tag.Query().Where(tag.Name(name)).Exist(ctx)
		if err != nil {
			return nil, rollback(tx, fmt.Errorf("failed to check for existing tag %q: %w", name, err))
		}
		if exists {
			return nil, rollback(tx, fmt.Errorf("generated group name %q collides with an existing tag, try again", name))
		}
		t, err := tx.Tag.Create().
			SetName(name).
			SetKind(tag.KindService).
			AddHostIDs(byCluster[id]...).
			Save(ctx)
		if err != nil {
			return nil, rollback(tx, fmt.Errorf("failed to create tag %q: %w", name, err))
		}
		createdIDs = append(createdIDs, t.ID)
	}
	if err := tx.Commit(); err != nil {
		return nil, rollback(tx, fmt.Errorf("failed to commit transaction: %w", err))
	}

	// Re-fetch outside the transaction: ents created on a committed tx stay
	// bound to it, which would break resolving edges (e.g. hosts) on the response.
	return client.Tag.Query().Where(tag.IDIn(createdIDs...)).Order(ent.Asc(tag.FieldID)).All(ctx)
}

// clusterHostsByRole picks each host's earliest "Process list" task, builds
// feature documents and clusters them -- with the auto-selected k, or the
// caller-pinned k if non-nil. It returns parallel slices: the host ids and
// the cluster label assigned to each.
func clusterHostsByRole(tasks []*ent.Task, k *int) (hostIDs []int, labels []int, err error) {
	if len(tasks) == 0 {
		return nil, nil, fmt.Errorf("no %q tasks found -- nothing to group", autogroupTomeName)
	}

	// Each host's FIRST (earliest-CREATED) matching task, for the
	// cleanest/least-drifted-from-initial-state process snapshot. Ordered
	// by CreatedAt specifically, not ExecFinishedAt: a task created early
	// but slow to execute can finish later than a task created after it,
	// so comparing finish times can pick a LATER-created task as "first".
	first := map[int]*ent.Task{}
	for _, t := range tasks {
		b := t.Edges.Beacon
		if b == nil || b.Edges.Host == nil {
			continue
		}
		id := b.Edges.Host.ID
		if cur, ok := first[id]; !ok || t.CreatedAt.Before(cur.CreatedAt) {
			first[id] = t
		}
	}

	ids := make([]int, 0, len(first))
	for id := range first {
		ids = append(ids, id)
	}
	sort.Ints(ids) // deterministic input order

	docs := make([]features.Doc, 0, len(ids))
	for _, id := range ids {
		t := first[id]
		h := t.Edges.Beacon.Edges.Host
		procs := make([]features.Process, 0, len(t.Edges.ReportedProcesses))
		for _, p := range t.Edges.ReportedProcesses {
			procs = append(procs, features.Process{Name: p.Name, Path: p.Path})
		}
		docs = append(docs, features.BuildDoc(features.Host{
			ID:         fmt.Sprint(h.ID),
			Identifier: h.Identifier,
			Name:       h.Name,
			PrimaryIP:  h.PrimaryIP,
			ExternalIP: h.ExternalIP,
			Platform:   h.Platform.String(),
			Principal:  t.Edges.Beacon.Principal,
		}, procs))
	}

	minHosts := autogroup.MinHosts
	if k != nil {
		// A pinned k only needs to be smaller than the host count, even if
		// that's below the auto-k search floor.
		minHosts = *k + 1
	}
	if len(docs) < minHosts {
		return nil, nil, fmt.Errorf("only %d host(s) with %q data -- too few to cluster meaningfully (need at least %d)",
			len(docs), autogroupTomeName, minHosts)
	}

	X := autogroup.Vectorize(docs, autogroup.DefaultSkip(false))
	if k != nil {
		labels, _ = autogroup.ClusterK(X, *k)
	} else {
		_, labels, _ = autogroup.ChooseKAuto(X)
	}
	if len(labels) != len(ids) {
		return nil, nil, fmt.Errorf("clustering failed to produce a grouping for %d host(s)", len(ids))
	}
	return ids, labels, nil
}
