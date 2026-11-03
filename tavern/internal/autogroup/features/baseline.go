package features

import (
	_ "embed"
	"encoding/json"
	"regexp"
	"strings"
)

// baseline_processes.json is the >=90%-of-hosts-per-platform noise list,
// computed once from our combined historical competition dumps (ists,
// rmccdc, seccdc, nccdc -- see tavern/internal/autogroup's doc comments
// for how/why). Each entry's "process" field is in the exact same format
// processFeatureToken produces (full exe path if the source process had
// one, lowercased; otherwise the bare process name, lowercased), so it
// can be compared directly against ProcPath tokens.
//
//go:embed baseline_processes.json
var baselineProcessesJSON []byte

type baselineEntry struct {
	Process string `json:"process"`
}

// baselineByPlatform maps a Host.Platform string (e.g. "PLATFORM_LINUX")
// to the set of process tokens considered baseline noise for that
// platform. Platforms absent from the embedded data (e.g. macOS never
// appeared in any of our dumps) simply have an empty/nil set, so nothing
// is filtered for them.
var baselineByPlatform = mustLoadBaseline(baselineProcessesJSON)

func mustLoadBaseline(data []byte) map[string]map[string]bool {
	var raw map[string][]baselineEntry
	if err := json.Unmarshal(data, &raw); err != nil {
		panic("features: failed to parse embedded baseline_processes.json: " + err.Error())
	}
	out := make(map[string]map[string]bool, len(raw))
	for platform, entries := range raw {
		set := make(map[string]bool, len(entries))
		for _, e := range entries {
			set[e.Process] = true
		}
		out[platform] = set
	}
	return out
}

// kernelThreadBaseNames are well-known Linux kernel-thread basenames
// (kthreadd's descendants: PPID 2, and never have an exe path), after
// stripping a trailing per-CPU/per-queue/per-device numeric suffix. This
// is intentionally broader than the frequency-based baselineByPlatform
// table above: a given kworker's exact per-CPU/per-queue suffix (e.g.
// "kworker/0:1H", "cpuhp/37") is host-specific -- it depends on that
// host's core count and attached devices/IRQs -- so it will never itself
// clear a 90% cross-host frequency bar the way "ksoftirqd/0" (present on
// every host, since every host has a CPU 0) does. Every kworker/irq/etc.
// is still noise regardless of which CPU or device it's pinned to, hence
// the separate name-pattern rule instead of relying on frequency alone.
var kernelThreadBaseNames = map[string]bool{
	"kthreadd": true, "kworker": true, "ksoftirqd": true, "migration": true,
	"cpuhp": true, "idle_inject": true, "watchdog": true, "watchdogd": true,
	"rcu_gp": true, "rcu_par_gp": true, "rcu_sched": true, "rcu_bh": true,
	"rcu_preempt": true, "rcu_tasks": true, "rcu_tasks_rude_": true, "rcu_tasks_trace": true,
	"kdevtmpfs": true, "netns": true, "kauditd": true, "khungtaskd": true,
	"oom_reaper": true, "writeback": true, "kcompactd": true, "ksmd": true,
	"khugepaged": true, "kintegrityd": true, "kblockd": true,
	"blkcg_punt_bio": true, "tpm_dev_wait": true, "ata_sff": true,
	"edac-poller": true, "devfreq_wq": true, "kswapd": true,
	"ecryptfs-kthrea": true, "kthrotld": true, "scsi_eh": true, "scsi_tmf": true,
	"ipv6_addrconf": true, "kstrp": true, "nvme-wq": true, "nvme-reset-wq": true,
	"nvme-delete-wq": true, "irq": true, "jbd2": true, "kjournald2": true,
	"dmcrypt_write": true, "xfsalloc": true, "xfs_mru_cache": true,
	"charger_manager": true, "deferwq": true,
}

var (
	kthreadBracketRe = regexp.MustCompile(`^\[(.+)\]$`)
	// a trailing per-CPU/per-queue/per-device suffix: "/<digits>" optionally
	// followed by ":<digits><H|I>?" (kworker's "/0:1H" style) and/or a
	// "-wordwordword" device-name tail ("irq/24-aerdrv"), OR a bare
	// trailing run of digits with no slash ("kswapd0", "cpuhp0").
	kthreadSuffixRe = regexp.MustCompile(`(/\d+(:\d+[HI]?)?(-[a-z]+(_[a-z]+)*)?|[0-9]+)$`)
)

// isLinuxKernelThread reports whether name (expected to already be
// lowercased, and only ever meaningful when the source process had no
// exe path -- kernel threads never have one) is a Linux kernel thread,
// after stripping bracket decoration ("[kthreadd]") and a per-CPU/
// per-queue/per-device numeric suffix ("ksoftirqd/0", "kworker/0:1h",
// "irq/24-aerdrv", "kswapd0").
func isLinuxKernelThread(name string) bool {
	name = strings.TrimSpace(name)
	if name == "" {
		return false
	}
	if m := kthreadBracketRe.FindStringSubmatch(name); m != nil {
		name = m[1]
	}
	if kernelThreadBaseNames[name] {
		return true
	}
	if base := kthreadSuffixRe.ReplaceAllString(name, ""); base != name {
		return kernelThreadBaseNames[base]
	}
	return false
}

// isBaselineNoise reports whether tok -- processFeatureToken's output for
// one process -- is noise to drop before building ProcPath, for a host on
// the given platform. hadPath indicates whether that process actually had
// an exe path (false means tok is the bare-name fallback).
func isBaselineNoise(platform, tok string, hadPath bool) bool {
	if baselineByPlatform[platform][tok] {
		return true
	}
	// Kernel threads never have an exe path; only consider the pattern
	// rule when we're looking at the bare-name fallback, and only on
	// Linux (the only platform this naming convention applies to).
	if !hadPath && platform == "PLATFORM_LINUX" && isLinuxKernelThread(tok) {
		return true
	}
	return false
}
