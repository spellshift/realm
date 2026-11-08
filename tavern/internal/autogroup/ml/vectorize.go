// Package ml implements the small amount of vectorization + clustering
// needed to group hosts by role signature, with no external numerical
// dependency. It mirrors the (interactively validated) reference
// implementation: per-block TF-IDF/binary vectorizers concatenated into
// one feature vector, then k-means with Euclidean distance and k chosen
// by silhouette score over a grid -- which, on real data, outperformed
// both a cosine/L2-normalized variant and HDBSCAN, so it's what's used
// here rather than something fancier.
package ml

import (
	"math"
	"sort"
)

// TFIDFVectorizer fits a vocabulary (capped at maxFeatures, dropping terms
// seen in fewer than minDF documents) and transforms bag-of-tokens
// documents into L2-normalized TF-IDF vectors, matching
// sklearn.feature_extraction.text.TfidfVectorizer's default behavior
// (smooth idf, L2 row norm) closely enough for this use case.
type TFIDFVectorizer struct {
	MinDF       int
	MaxFeatures int

	vocab []string       // index -> term
	index map[string]int // term -> index
	idf   []float64      // index -> idf weight
}

func (v *TFIDFVectorizer) Fit(docs []map[string]int) {
	df := map[string]int{}
	tf := map[string]int{}
	for _, d := range docs {
		for term, c := range d {
			df[term]++
			tf[term] += c
		}
	}

	type termCount struct {
		term  string
		df    int
		total int
	}
	var candidates []termCount
	for term, count := range df {
		if count >= v.MinDF {
			candidates = append(candidates, termCount{term, count, tf[term]})
		}
	}
	sort.Slice(candidates, func(i, j int) bool {
		if candidates[i].total != candidates[j].total {
			return candidates[i].total > candidates[j].total
		}
		return candidates[i].term < candidates[j].term
	})
	if v.MaxFeatures > 0 && len(candidates) > v.MaxFeatures {
		candidates = candidates[:v.MaxFeatures]
	}
	// stable column order
	sort.Slice(candidates, func(i, j int) bool { return candidates[i].term < candidates[j].term })

	v.vocab = make([]string, len(candidates))
	v.index = make(map[string]int, len(candidates))
	v.idf = make([]float64, len(candidates))
	n := float64(len(docs))
	for i, c := range candidates {
		v.vocab[i] = c.term
		v.index[c.term] = i
		// smooth idf, matching sklearn's default: ln((1+n)/(1+df)) + 1
		v.idf[i] = math.Log((1+n)/(1+float64(c.df))) + 1
	}
}

func (v *TFIDFVectorizer) Dim() int { return len(v.vocab) }

func (v *TFIDFVectorizer) Transform(d map[string]int) []float64 {
	out := make([]float64, len(v.vocab))
	for term, count := range d {
		if idx, ok := v.index[term]; ok {
			out[idx] = float64(count) * v.idf[idx]
		}
	}
	l2Normalize(out)
	return out
}

// BinaryVectorizer fits a vocabulary (dropping terms seen in fewer than
// minDF documents, optionally capped) and transforms documents into
// 0/1 presence vectors -- no idf weighting, no row normalization,
// matching sklearn's CountVectorizer(binary=True) as used for the
// IP-octet and external-IP "site" feature blocks.
type BinaryVectorizer struct {
	MinDF       int
	MaxFeatures int

	vocab []string
	index map[string]int
}

func (v *BinaryVectorizer) Fit(docs []map[string]int) {
	df := map[string]int{}
	for _, d := range docs {
		for term := range d {
			df[term]++
		}
	}
	type termCount struct {
		term string
		df   int
	}
	var candidates []termCount
	for term, count := range df {
		if count >= v.MinDF {
			candidates = append(candidates, termCount{term, count})
		}
	}
	sort.Slice(candidates, func(i, j int) bool {
		if candidates[i].df != candidates[j].df {
			return candidates[i].df > candidates[j].df
		}
		return candidates[i].term < candidates[j].term
	})
	if v.MaxFeatures > 0 && len(candidates) > v.MaxFeatures {
		candidates = candidates[:v.MaxFeatures]
	}
	sort.Slice(candidates, func(i, j int) bool { return candidates[i].term < candidates[j].term })

	v.vocab = make([]string, len(candidates))
	v.index = make(map[string]int, len(candidates))
	for i, c := range candidates {
		v.vocab[i] = c.term
		v.index[c.term] = i
	}
}

func (v *BinaryVectorizer) Dim() int { return len(v.vocab) }

func (v *BinaryVectorizer) Transform(d map[string]int) []float64 {
	out := make([]float64, len(v.vocab))
	for term := range d {
		if idx, ok := v.index[term]; ok {
			out[idx] = 1
		}
	}
	return out
}

// OneHotVectorizer encodes a single categorical value (e.g. platform)
// against a fixed vocabulary seen at fit time; unseen values at
// transform time map to the all-zero vector (handle_unknown="ignore").
type OneHotVectorizer struct {
	index map[string]int
	n     int
}

func (v *OneHotVectorizer) Fit(values []string) {
	v.index = map[string]int{}
	seen := map[string]bool{}
	var ordered []string
	for _, val := range values {
		if !seen[val] {
			seen[val] = true
			ordered = append(ordered, val)
		}
	}
	sort.Strings(ordered)
	for i, val := range ordered {
		v.index[val] = i
	}
	v.n = len(ordered)
}

func (v *OneHotVectorizer) Dim() int { return v.n }

func (v *OneHotVectorizer) Transform(value string) []float64 {
	out := make([]float64, v.n)
	if idx, ok := v.index[value]; ok {
		out[idx] = 1
	}
	return out
}

func l2Normalize(v []float64) {
	var sumSq float64
	for _, x := range v {
		sumSq += x * x
	}
	if sumSq == 0 {
		return
	}
	norm := math.Sqrt(sumSq)
	for i := range v {
		v[i] /= norm
	}
}

// Concat concatenates row-blocks into one feature matrix.
func Concat(blocks ...[][]float64) [][]float64 {
	n := len(blocks[0])
	out := make([][]float64, n)
	for i := 0; i < n; i++ {
		var row []float64
		for _, b := range blocks {
			row = append(row, b[i]...)
		}
		out[i] = row
	}
	return out
}
