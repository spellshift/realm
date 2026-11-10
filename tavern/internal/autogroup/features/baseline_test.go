package features

import (
	"testing"

	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
)

func TestIsLinuxKernelThreadMatchesKnownPatterns(t *testing.T) {
	for _, name := range []string{
		"kthreadd", "ksoftirqd/0", "ksoftirqd/15", "kworker/0:1H", "kworker/2:3",
		"cpuhp/0", "migration/1", "kswapd0", "kcompactd0", "kdevtmpfs",
		"oom_reaper", "irq/24-aerdrv", "[kthreadd]", "rcu_sched",
	} {
		if !isLinuxKernelThread(name) {
			t.Errorf("expected %q to be recognized as a Linux kernel thread", name)
		}
	}
}

func TestIsLinuxKernelThreadRejectsRealPrograms(t *testing.T) {
	for _, name := range []string{
		"sshd", "nginx", "systemd-journald", "udevadm", "cron", "php-fpm",
		"keepalived", "mysqld",
	} {
		if isLinuxKernelThread(name) {
			t.Errorf("expected %q to NOT be treated as a kernel thread", name)
		}
	}
}

func TestIsBaselineNoiseMatchesFrequencyList(t *testing.T) {
	// /usr/sbin/sshd is on the BSD baseline list at 100% host coverage.
	if !isBaselineNoise("PLATFORM_BSD", "/usr/sbin/sshd", true) {
		t.Error("expected /usr/sbin/sshd on PLATFORM_BSD to be baseline noise")
	}
	// The same token on an unrelated/unknown platform should NOT be
	// filtered -- the baseline is platform-specific.
	if isBaselineNoise("PLATFORM_MACOS", "/usr/sbin/sshd", true) {
		t.Error("did not expect /usr/sbin/sshd to be filtered for a platform with no baseline data")
	}
}

func TestIsBaselineNoiseCatchesKthreadsNotOnFrequencyList(t *testing.T) {
	// A per-CPU kworker suffix is host-specific and will never itself
	// clear the 90% frequency bar, but it's still noise.
	if !isBaselineNoise("PLATFORM_LINUX", "kworker/7:2", false) {
		t.Error("expected an arbitrary kworker to be filtered as noise on Linux even if absent from the frequency list")
	}
}

func TestIsBaselineNoiseKthreadRuleOnlyAppliesToLinuxBareNames(t *testing.T) {
	// Same literal name, but it came with a real exe path -- not a kernel
	// thread (kernel threads never have one), so it must NOT be filtered
	// by the kthread rule (though it could still be filtered if it also
	// happens to be on the frequency list, which "totally-made-up-kworker"
	// is not).
	if isBaselineNoise("PLATFORM_LINUX", "/opt/totally-made-up-kworker/bin", true) {
		t.Error("did not expect a process with a real exe path to be caught by the kthread name rule")
	}
	// Same bare name, but on a non-Linux platform -- the kthread naming
	// convention doesn't apply there.
	if isBaselineNoise("PLATFORM_WINDOWS", "kworker/7:2", false) {
		t.Error("did not expect the Linux kthread rule to apply on PLATFORM_WINDOWS")
	}
}

func TestBuildDocFiltersBaselineProcesses(t *testing.T) {
	procs := []*ent.HostProcess{
		{Name: "sshd", Path: "/usr/sbin/sshd"},   // real role signal elsewhere, but BSD-baseline
		{Name: "nginx", Path: "/usr/sbin/nginx"}, // NOT baseline -- should survive
		{Name: "ksoftirqd/0", Path: ""},          // kthread by pattern
		{Name: "kworker/3:1", Path: ""},          // kthread, not even on the frequency list
	}
	d := BuildDoc(&ent.Host{ID: 1, Platform: c2pb.Host_PLATFORM_LINUX}, "", procs)

	if _, ok := d.ProcPath["ksoftirqd/0"]; ok {
		t.Errorf("expected ksoftirqd/0 to be filtered out, got %v", d.ProcPath)
	}
	if _, ok := d.ProcPath["kworker/3:1"]; ok {
		t.Errorf("expected kworker/3:1 to be filtered out, got %v", d.ProcPath)
	}
	if d.ProcPath["/usr/sbin/nginx"] != 1 {
		t.Errorf("expected nginx to survive filtering, got %v", d.ProcPath)
	}
}

func TestBuildDocBaselineFilterIsPlatformSpecific(t *testing.T) {
	// /usr/sbin/sshd is BSD-baseline, not Windows-baseline (Windows has no
	// such path at all) -- so on an unrecognized/different platform it
	// must survive.
	procs := []*ent.HostProcess{{Name: "sshd", Path: "/usr/sbin/sshd"}}
	d := BuildDoc(&ent.Host{ID: 1, Platform: c2pb.Host_PLATFORM_WINDOWS}, "", procs)
	if d.ProcPath["/usr/sbin/sshd"] != 1 {
		t.Errorf("expected /usr/sbin/sshd to survive on a platform where it isn't baseline, got %v", d.ProcPath)
	}
}
