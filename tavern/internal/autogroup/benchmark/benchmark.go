// Package benchmark scores host-autogroup's clustering against a past
// competition's own already-applied tags, read directly from a raw
// mysqldump backup.
//
// This is ONLY for benchmarking against a dump that already has real
// tags applied -- it is never the path used to actually tag a new,
// untagged engagement (that's `host-autogroup live`, which talks to a
// running server). Scoring here mirrors exactly what was validated
// interactively before this tool was written: find each host's first
// task against the named tome, take its reported processes, and compare
// clustering against the host's own single "service"-kind tag.
package benchmark

import (
	"fmt"
	"sort"
	"strings"

	"realm.pub/tavern/internal/autogroup/features"
	"realm.pub/tavern/internal/autogroup/sqldump"
)

// GroundTruthHost is one host with both its feature document and the
// real tag it was given during that competition.
type GroundTruthHost struct {
	Doc   features.Doc
	Label string // the host's single "service"-kind tag; "" if 0 or >1
}

// Load streams a mysqldump file and reconstructs, per host: its first
// task against the tome named tomeName (case-insensitive exact match, or
// "<repo>::<tomeName>" for community tomes), the processes that task
// reported, and the host's own single "service"-kind tag.
func Load(path, tomeName string) ([]GroundTruthHost, error) {
	schema, err := sqldump.ReadSchema(path)
	if err != nil {
		return nil, fmt.Errorf("read schema: %w", err)
	}
	for _, t := range []string{"tomes", "quests", "tasks", "host_processes", "hosts", "tags", "host_tags", "beacons"} {
		if _, ok := schema[t]; !ok {
			return nil, fmt.Errorf("dump has no table %q -- is this a Tavern mysqldump?", t)
		}
	}

	col := func(table, name string) int {
		for i, c := range schema[table] {
			if c == name {
				return i
			}
		}
		return -1
	}

	// --- pass state ---
	tomeNameLower := strings.ToLower(tomeName)
	processListTomeIDs := map[int64]bool{}

	questTome := map[int64]int64{} // quest id -> tome id
	processListQuestIDs := map[int64]bool{}

	taskQuest := map[int64]int64{}      // task id -> quest id (only for matching quests)
	taskTimestamp := map[int64]string{} // task id -> execFinishedAt or createdAt
	taskBeacon := map[int64]int64{}     // task id -> beacon id

	type procRow struct {
		pid, ppid  int64
		name, path string
	}
	// (hostID, taskID) -> processes, only for tasks that matched
	hostTaskProcs := map[[2]int64][]procRow{}

	beaconPrincipal := map[int64]string{} // beacon id -> principal

	type hostRow struct {
		identifier, name, primaryIP, externalIP, platform string
	}
	hosts := map[int64]hostRow{}

	type tagRow struct {
		name, kind string
	}
	tags := map[int64]tagRow{}
	hostTagIDs := map[int64][]int64{}

	tiID := col("tomes", "id")
	tiName := col("tomes", "name")

	qiID := col("quests", "id")
	qiTome := col("quests", "quest_tome")

	taID := col("tasks", "id")
	taCreated := col("tasks", "created_at")
	taExecFinished := col("tasks", "exec_finished_at")
	taQuest := col("tasks", "quest_tasks")
	taBeacon := col("tasks", "task_beacon")

	hpPid := col("host_processes", "pid")
	hpPpid := col("host_processes", "ppid")
	hpName := col("host_processes", "name")
	hpPath := col("host_processes", "path")
	hpHost := col("host_processes", "host_process_host")
	hpTask := col("host_processes", "task_reported_processes")

	beID := col("beacons", "id")
	bePrincipal := col("beacons", "principal")

	hoID := col("hosts", "id")
	hoIdentifier := col("hosts", "identifier")
	hoName := col("hosts", "name")
	hoPrimaryIP := col("hosts", "primary_ip")
	hoExternalIP := col("hosts", "external_ip")
	hoPlatform := col("hosts", "platform")

	tgID := col("tags", "id")
	tgName := col("tags", "name")
	tgKind := col("tags", "kind")

	htHost := col("host_tags", "host_id")
	htTag := col("host_tags", "tag_id")

	// mysqldump writes tables in ALPHABETICAL order, not dependency order
	// -- confirmed by inspection: `host_processes` appears before `tasks`,
	// and `tomes` appears after both `quests` and `tasks`. So this must
	// be three separate full-file passes, each only reading the tables
	// whose dependencies were fully resolved by the previous pass; a
	// single combined pass would (and, in an earlier version of this
	// code, silently did) filter host_processes rows against an
	// empty/incomplete taskTimestamp map just because of row order in
	// the file.
	//
	// pass 1: tomes + quests -> which quest ids used the named tome
	err = sqldump.ForEachRow(path, map[string]bool{"tomes": true, "quests": true}, schema, func(table string, row sqldump.Row) error {
		switch table {
		case "tomes":
			id := asInt64(row[tiID])
			name := strings.ToLower(strings.TrimSpace(asString(row[tiName])))
			if name == tomeNameLower || strings.HasSuffix(name, "::"+tomeNameLower) {
				processListTomeIDs[id] = true
			}
		case "quests":
			id := asInt64(row[qiID])
			tome := asInt64(row[qiTome])
			questTome[id] = tome
		}
		return nil
	})
	if err != nil {
		return nil, fmt.Errorf("pass 1 (tomes/quests): %w", err)
	}
	for q, tomeID := range questTome {
		if processListTomeIDs[tomeID] {
			processListQuestIDs[q] = true
		}
	}

	// pass 2: tasks -> which task ids qualify, and their timestamps
	err = sqldump.ForEachRow(path, map[string]bool{"tasks": true}, schema, func(table string, row sqldump.Row) error {
		id := asInt64(row[taID])
		quest := asInt64(row[taQuest])
		if !processListQuestIDs[quest] {
			return nil
		}
		taskQuest[id] = quest
		taskBeacon[id] = asInt64(row[taBeacon])
		ts := asString(row[taCreated])
		if row[taExecFinished] != nil {
			ts = asString(row[taExecFinished])
		}
		taskTimestamp[id] = ts
		return nil
	})
	if err != nil {
		return nil, fmt.Errorf("pass 2 (tasks): %w", err)
	}

	// pass 3: host_processes (filtered against the now-complete
	// taskTimestamp) + hosts + tags + host_tags + beacons, none of which
	// depend on each other so they're free to share one pass
	err = sqldump.ForEachRow(path, map[string]bool{"host_processes": true, "hosts": true, "tags": true, "host_tags": true, "beacons": true}, schema, func(table string, row sqldump.Row) error {
		switch table {
		case "host_processes":
			taskID := asInt64(row[hpTask])
			if !taskInQualifyingTask(taskID, taskTimestamp) {
				return nil
			}
			hostID := asInt64(row[hpHost])
			key := [2]int64{hostID, taskID}
			hostTaskProcs[key] = append(hostTaskProcs[key], procRow{
				pid: asInt64(row[hpPid]), ppid: asInt64(row[hpPpid]),
				name: asString(row[hpName]), path: asString(row[hpPath]),
			})
		case "beacons":
			beaconPrincipal[asInt64(row[beID])] = asString(row[bePrincipal])
		case "hosts":
			id := asInt64(row[hoID])
			hosts[id] = hostRow{
				identifier: asString(row[hoIdentifier]), name: asString(row[hoName]),
				primaryIP: asString(row[hoPrimaryIP]), externalIP: asString(row[hoExternalIP]),
				platform: asString(row[hoPlatform]),
			}
		case "tags":
			id := asInt64(row[tgID])
			tags[id] = tagRow{name: asString(row[tgName]), kind: asString(row[tgKind])}
		case "host_tags":
			h := asInt64(row[htHost])
			tg := asInt64(row[htTag])
			hostTagIDs[h] = append(hostTagIDs[h], tg)
		}
		return nil
	})
	if err != nil {
		return nil, fmt.Errorf("pass 3 (host_processes/hosts/tags/host_tags/beacons): %w", err)
	}

	// pick, per host, the processes from its EARLIEST qualifying task
	type firstTask struct {
		taskID    int64
		timestamp string
	}
	firstByHost := map[int64]firstTask{}
	for key := range hostTaskProcs {
		hostID, taskID := key[0], key[1]
		ts := taskTimestamp[taskID]
		if cur, ok := firstByHost[hostID]; !ok || ts < cur.timestamp {
			firstByHost[hostID] = firstTask{taskID, ts}
		}
	}

	var out []GroundTruthHost
	// deterministic order for reproducible benchmark runs
	hostIDs := make([]int64, 0, len(firstByHost))
	for id := range firstByHost {
		hostIDs = append(hostIDs, id)
	}
	sort.Slice(hostIDs, func(i, j int) bool { return hostIDs[i] < hostIDs[j] })

	for _, hostID := range hostIDs {
		h, ok := hosts[hostID]
		if !ok {
			continue
		}
		ft := firstByHost[hostID]
		procs := hostTaskProcs[[2]int64{hostID, ft.taskID}]

		var serviceTags []string
		for _, tagID := range hostTagIDs[hostID] {
			if tg, ok := tags[tagID]; ok && tg.kind == "service" {
				serviceTags = append(serviceTags, tg.name)
			}
		}
		label := ""
		if len(serviceTags) == 1 {
			label = serviceTags[0]
		}

		fProcs := make([]features.Process, 0, len(procs))
		for _, p := range procs {
			fProcs = append(fProcs, features.Process{Name: p.name, Path: p.path})
		}
		// principal comes from the beacon that ran this task, not from the
		// process list
		principal := beaconPrincipal[taskBeacon[ft.taskID]]
		doc := features.BuildDoc(features.Host{
			ID: fmt.Sprintf("%d", hostID), Identifier: h.identifier, Name: h.name,
			PrimaryIP: h.primaryIP, ExternalIP: h.externalIP, Platform: h.platform,
			Principal: principal,
		}, fProcs)

		out = append(out, GroundTruthHost{Doc: doc, Label: label})
	}

	return out, nil
}

func taskInQualifyingTask(taskID int64, taskTimestamp map[int64]string) bool {
	_, ok := taskTimestamp[taskID]
	return ok
}

func asInt64(v any) int64 {
	if v == nil {
		return 0
	}
	if i, ok := v.(int64); ok {
		return i
	}
	return 0
}

func asString(v any) string {
	if v == nil {
		return ""
	}
	if s, ok := v.(string); ok {
		return s
	}
	return fmt.Sprintf("%v", v)
}
