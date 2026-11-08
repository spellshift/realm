package ml

import "math"

// ClusteringAccuracy scores predicted cluster ids against ground-truth
// string labels by finding the best possible one-to-one renaming of
// cluster ids to labels (Hungarian/optimal assignment on the confusion
// matrix) and returning the fraction of points correct under that
// renaming. This is "if a human picks the single best name for each
// cluster, what fraction of hosts end up correctly grouped" -- the same
// metric used to validate this tool's approach against three past
// competitions' data.
func ClusteringAccuracy(trueLabels []string, predClusters []int) float64 {
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

	// confusion[pred][true] = count
	confusion := make([][]float64, len(predOrdered))
	for i := range confusion {
		confusion[i] = make([]float64, len(trueOrdered))
	}
	for i := range trueLabels {
		confusion[predIndex[predClusters[i]]][trueIndex[trueLabels[i]]]++
	}

	matched := maxWeightAssignment(confusion)
	return matched / float64(len(trueLabels))
}

// maxWeightAssignment finds the assignment of rows to columns (each used
// at most once) maximizing the sum of selected entries, via the
// Hungarian algorithm (Kuhn-Munkres, O(n^3)) on a cost matrix padded to
// square with zeros when rectangular -- extra rows/columns simply go
// unmatched, which is correct here since clusters/labels can outnumber
// each other.
func maxWeightAssignment(weights [][]float64) float64 {
	rows := len(weights)
	if rows == 0 {
		return 0
	}
	cols := len(weights[0])
	n := rows
	if cols > n {
		n = cols
	}

	// square cost matrix for MINIMIZATION: negate weights, pad with 0
	cost := make([][]float64, n)
	for i := 0; i < n; i++ {
		cost[i] = make([]float64, n)
		for j := 0; j < n; j++ {
			if i < rows && j < cols {
				cost[i][j] = -weights[i][j]
			}
		}
	}

	assignment := hungarian(cost)

	var total float64
	for i := 0; i < rows; i++ {
		j := assignment[i]
		if j >= 0 && j < cols {
			total += weights[i][j]
		}
	}
	return total
}

// hungarian solves the minimum-cost assignment problem on a square cost
// matrix (Jonker-Volgenant-free Kuhn-Munkres with potentials / shortest
// augmenting paths, O(n^3)). Returns, for each row, the column it was
// assigned to.
//
// This is the classic competitive-programming formulation operating on
// 1-indexed rows/columns internally; see e.g. cp-algorithms.com's writeup
// of the "assignment problem" for the derivation.
func hungarian(cost [][]float64) []int {
	n := len(cost)
	const inf = math.MaxFloat64 / 2

	u := make([]float64, n+1)
	v := make([]float64, n+1)
	p := make([]int, n+1) // p[j] = row assigned to column j (1-indexed), 0 = unassigned
	way := make([]int, n+1)

	for i := 1; i <= n; i++ {
		p[0] = i
		j0 := 0
		minv := make([]float64, n+1)
		used := make([]bool, n+1)
		for j := range minv {
			minv[j] = inf
		}

		for {
			used[j0] = true
			i0 := p[j0]
			delta := inf
			j1 := -1
			for j := 1; j <= n; j++ {
				if used[j] {
					continue
				}
				cur := cost[i0-1][j-1] - u[i0] - v[j]
				if cur < minv[j] {
					minv[j] = cur
					way[j] = j0
				}
				if minv[j] < delta {
					delta = minv[j]
					j1 = j
				}
			}
			for j := 0; j <= n; j++ {
				if used[j] {
					u[p[j]] += delta
					v[j] -= delta
				} else {
					minv[j] -= delta
				}
			}
			j0 = j1
			if p[j0] == 0 {
				break
			}
		}

		for j0 != 0 {
			j1 := way[j0]
			p[j0] = p[j1]
			j0 = j1
		}
	}

	rowToCol := make([]int, n)
	for i := range rowToCol {
		rowToCol[i] = -1
	}
	for j := 1; j <= n; j++ {
		if p[j] != 0 {
			rowToCol[p[j]-1] = j - 1
		}
	}
	return rowToCol
}
