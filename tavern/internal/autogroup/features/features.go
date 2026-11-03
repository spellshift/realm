// Package features builds per-host bag-of-tokens documents from a host's
// first "Process list" task, for unsupervised clustering into generic
// role groups within a single engagement.
//
// This mirrors the feature set validated interactively against three past
// competitions' data:
//   - full executable path of each process, kept as ONE atomic token (not
//     split into directory components) -- carries OS/install-location
//     context for free, and fixes a double-counting bug an earlier
//     name+path-basename version had
//   - the beacon's principal (the identity the agent is authenticated as
//     on that host, e.g. "root"/"SYSTEM"/a service account) -- a single
//     stable per-host value, NOT scanned from the process list (an
//     earlier version aggregated per-process principals, which let
//     whichever generic account happened to run the most processes
//     dominate the TF-IDF weight and drown out the actual signal)
//   - hostname, tokenized -- a real, always-available field that's often
//     deliberately descriptive of role; a small denylist strips
//     placeholder values (localhost, etc.) that would create false
//     matches between unrelated hosts
//   - IP octets (last + third) treated as CATEGORICAL lookups, not ordinal
//     numbers -- confirmed in real data that a fixed octet can denote a
//     role (e.g. all DNS boxes in one competition's network sat at .248)
//     but the convention is engagement/template-specific, not a universal
//     constant. Both the internal and external IP's octets are computed
//     here; main's defaultSkip picks internal by default or external
//     with --ext (never both -- adding external IP features on top of
//     internal measurably hurt clustering)
//   - full external IP as a "site" token -- computed but unused by
//     default (see defaultSkip): it hurt accuracy on both benchmark dumps
//   - platform (OS)
//
// Deliberately excluded: raw/absolute primary IP (doesn't generalize even
// within clustering since it's effectively a per-host identifier), process
// cmd/env/cwd (untested, likely noisy), process count (weak/unused).
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
var genericHostnames = map[string]bool{
	"localhost": true, "localhost.localdomain": true, "desktop": true,
	"pc": true, "host": true, "ubuntu": true, "debian": true,
	"centos": true, "windows": true, "unknown": true, "": true,
}

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
		incr(d.ProcPath, processFeatureToken(p.Name, p.Path))
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
