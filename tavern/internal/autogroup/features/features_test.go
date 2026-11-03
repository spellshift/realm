package features

import "testing"

func TestProcessFeatureTokenPrefersFullPath(t *testing.T) {
	got := processFeatureToken("sshd [listener] 0 of 10-100 startups", "/usr/sbin/sshd")
	want := "/usr/sbin/sshd"
	if got != want {
		t.Errorf("expected full path %q to win over dirty ps-style name, got %q", want, got)
	}
}

func TestProcessFeatureTokenFallsBackToCleanName(t *testing.T) {
	got := processFeatureToken("cron", "")
	if got != "cron" {
		t.Errorf("expected bare clean name when path is missing, got %q", got)
	}
}

func TestProcessFeatureTokenFallsBackToFirstWord(t *testing.T) {
	got := processFeatureToken("php-fpm: pool nginx", "")
	if got != "php-fpm:" {
		t.Errorf("expected first whitespace-split word as last resort, got %q", got)
	}
}

func TestProcessFeatureTokenHandlesWindowsPaths(t *testing.T) {
	got := processFeatureToken("svchost.exe", `C:\Windows\System32\svchost.exe`)
	want := `c:\windows\system32\svchost.exe`
	if got != want {
		t.Errorf("expected lowercased full Windows path preserved as one atomic token, got %q", got)
	}
}

func TestGenericHostnameIsFilteredOut(t *testing.T) {
	d := BuildDoc(Host{ID: "1", Name: "localhost"}, nil)
	if len(d.HostnameTok) != 0 {
		t.Errorf("expected generic hostname 'localhost' to produce zero tokens, got %v", d.HostnameTok)
	}
}

func TestRealHostnameIsTokenized(t *testing.T) {
	d := BuildDoc(Host{ID: "1", Name: "mail-server-01"}, nil)
	for _, want := range []string{"mail", "server", "01"} {
		if d.HostnameTok[want] == 0 {
			t.Errorf("expected token %q from hostname, got %v", want, d.HostnameTok)
		}
	}
}

func TestLoopbackExternalIPIsIgnored(t *testing.T) {
	d := BuildDoc(Host{ID: "1", PrimaryIP: "10.0.0.5", ExternalIP: "127.0.0.1"}, nil)
	if len(d.ExtIPOctet) != 0 {
		t.Errorf("expected loopback external IP to produce no octet tokens, got %v", d.ExtIPOctet)
	}
	if len(d.ExtIPFull) != 0 {
		t.Errorf("expected loopback external IP to produce no site token, got %v", d.ExtIPFull)
	}
}

func TestPrimaryIPOctetsAreCategoricalTokens(t *testing.T) {
	d := BuildDoc(Host{ID: "1", PrimaryIP: "10.250.104.248"}, nil)
	if d.IPOctet["oct4_248"] == 0 {
		t.Errorf("expected last-octet token oct4_248, got %v", d.IPOctet)
	}
	if d.IPOctet["oct3_104"] == 0 {
		t.Errorf("expected third-octet token oct3_104, got %v", d.IPOctet)
	}
}

func TestExternalIPFullIsItsOwnSiteToken(t *testing.T) {
	d := BuildDoc(Host{ID: "1", ExternalIP: "16.59.169.33"}, nil)
	if d.ExtIPFull["extip_16_59_169_33"] == 0 {
		t.Errorf("expected full external IP site token, got %v", d.ExtIPFull)
	}
}

func TestPrincipalComesFromHostNotProcessList(t *testing.T) {
	// many processes, all running as root -- principal must still be a
	// single presence entry for "root", not frequency-weighted, and must
	// come from the beacon's principal rather than being scanned off the
	// process list at all
	procs := []Process{
		{Name: "sshd", Path: "/usr/sbin/sshd"},
		{Name: "cron", Path: "/usr/sbin/cron"},
		{Name: "nginx", Path: "/usr/sbin/nginx"},
	}
	d := BuildDoc(Host{ID: "1", Principal: "root"}, procs)
	if len(d.Principal) != 1 || d.Principal["root"] != 1 {
		t.Errorf("expected exactly one presence entry for host principal 'root', got %v", d.Principal)
	}
}

func TestNoPrincipalProducesEmptyBlock(t *testing.T) {
	d := BuildDoc(Host{ID: "1"}, nil)
	if len(d.Principal) != 0 {
		t.Errorf("expected empty principal block when host has no beacon principal, got %v", d.Principal)
	}
}

func TestProcessTokensAccumulateFrequency(t *testing.T) {
	procs := []Process{
		{Name: "nginx", Path: "/usr/sbin/nginx"},
		{Name: "nginx", Path: "/usr/sbin/nginx"},
		{Name: "cron", Path: "/usr/sbin/cron"},
	}
	d := BuildDoc(Host{ID: "1"}, procs)
	if d.ProcPath["/usr/sbin/nginx"] != 2 {
		t.Errorf("expected nginx counted twice, got %v", d.ProcPath)
	}
	if d.ProcPath["/usr/sbin/cron"] != 1 {
		t.Errorf("expected cron counted once, got %v", d.ProcPath)
	}
}
