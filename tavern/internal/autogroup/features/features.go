// Package features builds per-host bag-of-tokens feature documents from a
// host's first "Process list" task, for unsupervised clustering into
// generic role groups within one engagement.
//
// Feature blocks: process executable path (full path, one atomic token),
// beacon principal, tokenized hostname, primary/external IP octets
// (categorical), external IP as a "site" token (unused by default, see
// DefaultSkip), and platform. Raw primary IP, process cmd/env/cwd, and
// process count are deliberately excluded.
//
// Baseline-noise processes are dropped from ProcPath before counting:
// a precomputed per-platform frequency list plus, on Linux, kernel
// threads matched by name pattern -- see baseline.go.
package features

import (
	"regexp"
	"strconv"
	"strings"
)

// Process is the subset of a reported HostProcess this package needs.
type Process struct {
	Name string
	Path string
}

// Host is the subset of a Host (plus its beacon's principal) this
// package needs.
type Host struct {
	ID         string
	Identifier string
	Name       string
	PrimaryIP  string
	ExternalIP string
	Platform   string
	// Principal is the BEACON's principal (Beacon.principal: "the
	// identity the beacon is authenticated as"), not a process-list
	// field -- one stable value per host.
	Principal string
}

// Doc is a host's feature document: one bag-of-tokens per feature block,
// kept separate so each block can be vectorized (and max-feature-capped)
// independently, matching the validated reference implementation.
type Doc struct {
	HostID     string
	Identifier string
	Hostname   string

	ProcPath    map[string]int // full exe path tokens (one per process instance)
	Principal   map[string]int // the beacon's principal (0 or 1 entries)
	HostnameTok map[string]int // tokenized hostname
	IPOctet     map[string]int // primary IP last/third octet, categorical
	ExtIPOctet  map[string]int // external IP last/third octet, categorical
	ExtIPFull   map[string]int // full external IP, as a "site" token
	Platform    string
}

var (
	tokenRe     = regexp.MustCompile(`[a-zA-Z0-9]+`)
	cleanNameRe = regexp.MustCompile(`^[A-Za-z0-9_.\-]+$`)
)

// genericHostnames are placeholder hostnames that carry no role signal and
// would create FALSE cross-host matches if tokenized naively (e.g. every
// WSL/container host reporting itself as "localhost" does not mean they
// share a role).
var genericHostnames = toSet(
	"localhost", "localhost.localdomain", "desktop", "pc", "host",
	"ubuntu", "debian", "centos", "windows", "unknown", "",
)

// genericIPs are loopback/unset addresses that tell us nothing about
// site/NAT structure.
var genericIPs = map[string]bool{"127.0.0.1": true, "0.0.0.0": true, "": true}

func incr(m map[string]int, k string) {
	if k == "" {
		return
	}
	m[k]++
}

// processFeatureToken returns a single atomic feature token for a process:
// the FULL executable path when available (kept whole -- not split into
// directory components, and not just the basename), falling back to the
// bare process name only when path is missing.
func processFeatureToken(name, path string) string {
	path = strings.TrimSpace(path)
	if path != "" {
		return strings.ReplaceAll(strings.ToLower(path), " ", "_")
	}
	name = strings.TrimSpace(name)
	if name != "" && cleanNameRe.MatchString(name) {
		return strings.ToLower(name)
	}
	if name != "" {
		return strings.ToLower(strings.Fields(name)[0])
	}
	return ""
}

func tokenize(s string) []string {
	return tokenRe.FindAllString(strings.ToLower(s), -1)
}

// octetSplit returns (lastOctet, thirdOctet) tokens for an IPv4 address,
// or ("","") if the address is missing/generic/malformed.
func octetSplit(prefix, ip string) (last, third string) {
	if ip == "" || genericIPs[ip] {
		return "", ""
	}
	parts := strings.Split(ip, ".")
	if len(parts) != 4 {
		return "", ""
	}
	if _, err := strconv.Atoi(parts[3]); err == nil {
		last = prefix + "oct4_" + parts[3]
	}
	if _, err := strconv.Atoi(parts[2]); err == nil {
		third = prefix + "oct3_" + parts[2]
	}
	return last, third
}

// BuildDoc builds the feature document for one host from its first
// process-list snapshot.
func BuildDoc(h Host, procs []Process) Doc {
	d := Doc{
		HostID:      h.ID,
		Identifier:  h.Identifier,
		Hostname:    h.Name,
		ProcPath:    map[string]int{},
		Principal:   map[string]int{},
		HostnameTok: map[string]int{},
		IPOctet:     map[string]int{},
		ExtIPOctet:  map[string]int{},
		ExtIPFull:   map[string]int{},
		Platform:    h.Platform,
	}

	for _, p := range procs {
		tok := processFeatureToken(p.Name, p.Path)
		hadPath := strings.TrimSpace(p.Path) != ""
		if isBaselineNoise(h.Platform, tok, hadPath) {
			continue
		}
		incr(d.ProcPath, tok)
	}
	if h.Principal != "" {
		d.Principal[strings.ToLower(h.Principal)] = 1
	}

	hostnameLower := strings.ToLower(strings.TrimSpace(h.Name))
	if !genericHostnames[hostnameLower] {
		for _, t := range tokenize(h.Name) {
			incr(d.HostnameTok, t)
		}
	}

	last, third := octetSplit("", h.PrimaryIP)
	incr(d.IPOctet, last)
	incr(d.IPOctet, third)

	extLast, extThird := octetSplit("ext_", h.ExternalIP)
	incr(d.ExtIPOctet, extLast)
	incr(d.ExtIPOctet, extThird)

	if h.ExternalIP != "" && !genericIPs[h.ExternalIP] {
		incr(d.ExtIPFull, "extip_"+strings.ReplaceAll(h.ExternalIP, ".", "_"))
	}

	return d
}
