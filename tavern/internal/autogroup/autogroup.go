// Package autogroup implements the clustering engine that groups a single
// engagement's hosts into generic role buckets from each host's first
// "Process list" task, so a human can name each cluster once instead of
// tagging every host by hand.
//
// It backs the Tavern server's runHostAutogroup mutation. Everything here is
// fit fresh on each run's own data; nothing trains on or reuses data from a
// past engagement.
package autogroup

import (
	"time"

	"realm.pub/tavern/internal/autogroup/features"
	"realm.pub/tavern/internal/autogroup/ml"
)

// Tunables validated against real past-engagement data.
const (
	MinDF           = 2
	ProcPathMaxDim  = 400
	PrincipalMaxDim = 100
	HostnameMaxDim  = 100

	KMin, KMax, KStep = 4, 40, 2
	NInit             = 10
	MaxIter           = 100
)

// MinHosts is the fewest hosts with process-list data this package will
// cluster meaningfully (the auto-k grid search starts at KMin groups).
const MinHosts = KMin

// Vectorize fits each feature block's vectorizer on the whole corpus and
// concatenates the results -- no cross-engagement training, everything is
// fit fresh on this run's own data.
//
// skip names feature blocks to leave out entirely ("procpath", "principal",
// "hostname", "ipoctet", "extipoctet", "extipfull", "platform"); nil/empty
// uses every block.
func Vectorize(docs []features.Doc, skip map[string]bool) [][]float64 {
	procPathDocs := make([]map[string]int, len(docs))
	principalDocs := make([]map[string]int, len(docs))
	hostnameDocs := make([]map[string]int, len(docs))
	ipOctetDocs := make([]map[string]int, len(docs))
	extIPOctetDocs := make([]map[string]int, len(docs))
	extIPFullDocs := make([]map[string]int, len(docs))
	platforms := make([]string, len(docs))
	for i, d := range docs {
		procPathDocs[i] = d.ProcPath
		principalDocs[i] = d.Principal
		hostnameDocs[i] = d.HostnameTok
		ipOctetDocs[i] = d.IPOctet
		extIPOctetDocs[i] = d.ExtIPOctet
		extIPFullDocs[i] = d.ExtIPFull
		platforms[i] = d.Platform
	}

	procPathVec := &ml.TFIDFVectorizer{MinDF: MinDF, MaxFeatures: ProcPathMaxDim}
	procPathVec.Fit(procPathDocs)
	principalVec := &ml.TFIDFVectorizer{MinDF: MinDF, MaxFeatures: PrincipalMaxDim}
	principalVec.Fit(principalDocs)
	hostnameVec := &ml.TFIDFVectorizer{MinDF: MinDF, MaxFeatures: HostnameMaxDim}
	hostnameVec.Fit(hostnameDocs)
	ipOctetVec := &ml.BinaryVectorizer{MinDF: MinDF}
	ipOctetVec.Fit(ipOctetDocs)
	extIPOctetVec := &ml.BinaryVectorizer{MinDF: MinDF}
	extIPOctetVec.Fit(extIPOctetDocs)
	extIPFullVec := &ml.BinaryVectorizer{MinDF: 1}
	extIPFullVec.Fit(extIPFullDocs)
	platVec := &ml.OneHotVectorizer{}
	platVec.Fit(platforms)

	// name -> per-row transform; order here is the column order
	type block struct {
		name string
		fn   func(i int) []float64
	}
	all := []block{
		{"procpath", func(i int) []float64 { return procPathVec.Transform(procPathDocs[i]) }},
		{"principal", func(i int) []float64 { return principalVec.Transform(principalDocs[i]) }},
		{"hostname", func(i int) []float64 { return hostnameVec.Transform(hostnameDocs[i]) }},
		{"ipoctet", func(i int) []float64 { return ipOctetVec.Transform(ipOctetDocs[i]) }},
		{"extipoctet", func(i int) []float64 { return extIPOctetVec.Transform(extIPOctetDocs[i]) }},
		{"extipfull", func(i int) []float64 { return extIPFullVec.Transform(extIPFullDocs[i]) }},
		{"platform", func(i int) []float64 { return platVec.Transform(platforms[i]) }},
	}

	var blocks [][][]float64
	for _, b := range all {
		if skip[b.name] {
			continue
		}
		rows := make([][]float64, len(docs))
		for i := range docs {
			rows[i] = b.fn(i)
		}
		blocks = append(blocks, rows)
	}
	if len(blocks) == 0 {
		return nil
	}
	return ml.Concat(blocks...)
}

// DefaultSkip is the feature set used out of the box.
//
// External-IP features are off by default: leaving out extipoctet and
// extipfull gained +7 to +10 points of oracle-k accuracy (and +3 to +9
// auto-k) on both benchmark dumps, consistently across both. The two
// IP-derived blocks are alternatives, not additive: by default the
// INTERNAL IP's octets are used; with ext=true the EXTERNAL IP's octets
// are used instead, for networks where the internal address is useless
// because it's been remapped/NATed behind an external one. The full
// external IP ("site" token) is never used.
func DefaultSkip(ext bool) map[string]bool {
	if ext {
		return map[string]bool{"ipoctet": true, "extipfull": true}
	}
	return map[string]bool{"extipoctet": true, "extipfull": true}
}

// ChooseKAuto runs the auto-k silhouette grid search (KMin..KMax, step
// KStep), clamping KMax below the number of hosts so k never reaches n.
//
// Seeded from the current time on each call rather than a fixed constant:
// this is a one-shot admin action (runHostAutogroup), not a benchmark
// needing byte-for-byte reproducibility, so a frozen seed would mean
// every call on the same data forever explores the exact same nInit
// random restarts -- if that happens to land on a mediocre local optimum,
// there'd be no way to get a different grouping by simply trying again.
func ChooseKAuto(X [][]float64) (k int, labels []int, silhouette float64) {
	kMax := KMax
	if kMax >= len(X) {
		kMax = len(X) - 1
	}
	return ml.ChooseK(X, KMin, kMax, KStep, NInit, MaxIter, time.Now().UnixNano())
}

// ClusterK runs k-means for a caller-pinned k instead of auto-selecting
// it. The caller is responsible for validating 0 < k < len(X) first. See
// ChooseKAuto's doc comment for why the seed is time-based, not fixed.
func ClusterK(X [][]float64, k int) (labels []int, silhouette float64) {
	labels, _ = ml.KMeans(X, k, NInit, MaxIter, time.Now().UnixNano())
	return labels, ml.SilhouetteScore(X, labels)
}
