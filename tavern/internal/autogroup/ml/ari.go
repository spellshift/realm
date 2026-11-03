package ml

// AdjustedRandIndex computes the Adjusted Rand Index between a clustering
// and ground-truth labels, matching
// sklearn.metrics.adjusted_rand_score: 1.0 for a perfect match (up to
// relabeling), ~0.0 for a random/chance assignment, negative for
// worse-than-chance agreement.
func AdjustedRandIndex(trueLabels []string, predClusters []int) float64 {
	trueIndex := map[string]int{}
	var trueOrdered []string
	for _, t := range trueLabels {
		if _, ok := trueIndex[t]; !ok {
			trueIndex[t] = len(trueOrdered)
			trueOrdered = append(trueOrdered, t)
		}
	}
	predIndex := map[int]int{}
	var predOrdered []int
	for _, p := range predClusters {
		if _, ok := predIndex[p]; !ok {
			predIndex[p] = len(predOrdered)
			predOrdered = append(predOrdered, p)
		}
	}

	contingency := make([][]float64, len(predOrdered))
	for i := range contingency {
		contingency[i] = make([]float64, len(trueOrdered))
	}
	n := float64(len(trueLabels))
	for i := range trueLabels {
		contingency[predIndex[predClusters[i]]][trueIndex[trueLabels[i]]]++
	}

	rowSums := make([]float64, len(predOrdered))
	colSums := make([]float64, len(trueOrdered))
	var sumComb float64
	for i, row := range contingency {
		for j, v := range row {
			rowSums[i] += v
			colSums[j] += v
			sumComb += comb2(v)
		}
	}

	var sumRowComb, sumColComb float64
	for _, a := range rowSums {
		sumRowComb += comb2(a)
	}
	for _, b := range colSums {
		sumColComb += comb2(b)
	}

	totalComb := comb2(n)
	if totalComb == 0 {
		return 1
	}
	expectedIndex := sumRowComb * sumColComb / totalComb
	maxIndex := (sumRowComb + sumColComb) / 2
	denom := maxIndex - expectedIndex
	if denom == 0 {
		// every point in its own cluster, or one giant cluster on both
		// sides -- sklearn treats this as a perfect/degenerate match
		if sumComb == expectedIndex {
			return 1
		}
		return 0
	}
	return (sumComb - expectedIndex) / denom
}

func comb2(n float64) float64 {
	if n < 2 {
		return 0
	}
	return n * (n - 1) / 2
}
