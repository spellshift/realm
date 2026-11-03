package ml

import (
	"math"
	"math/rand"
)

func euclidean(a, b []float64) float64 {
	var sum float64
	for i := range a {
		d := a[i] - b[i]
		sum += d * d
	}
	return math.Sqrt(sum)
}

func nearestCentroid(point []float64, centroids [][]float64) (int, float64) {
	best, bestDist := -1, math.Inf(1)
	for i, c := range centroids {
		d := euclidean(point, c)
		if d < bestDist {
			bestDist = d
			best = i
		}
	}
	return best, bestDist
}

// kmeansPlusPlusInit seeds k centroids using GREEDY k-means++ (the variant
// scikit-learn uses): each step samples several candidate centers with
// probability proportional to squared distance from the nearest
// already-chosen centroid, then keeps whichever candidate lowers the total
// potential the most. Plain single-candidate k-means++ was measurably
// worse on identical input (an earlier version of this function: 0.80 vs
// sklearn's 0.91 clustering accuracy on the same feature matrix), because
// one unlucky draw can strand two real groups under a single centroid.
func kmeansPlusPlusInit(X [][]float64, k int, rng *rand.Rand) [][]float64 {
	n := len(X)
	nTrials := 2 + int(math.Log(float64(k)))

	centroids := make([][]float64, 0, k)
	centroids = append(centroids, append([]float64{}, X[rng.Intn(n)]...))

	// closestSq[i] = squared distance from point i to its nearest chosen centroid
	closestSq := make([]float64, n)
	var potential float64
	for i, x := range X {
		d := euclidean(x, centroids[0])
		closestSq[i] = d * d
		potential += closestSq[i]
	}

	for len(centroids) < k {
		if potential == 0 {
			// all remaining points coincide with existing centroids
			centroids = append(centroids, append([]float64{}, X[rng.Intn(n)]...))
			continue
		}

		bestIdx := -1
		bestPotential := math.Inf(1)
		var bestClosest []float64

		for t := 0; t < nTrials; t++ {
			target := rng.Float64() * potential
			var cum float64
			cand := n - 1
			for i, d := range closestSq {
				cum += d
				if cum >= target {
					cand = i
					break
				}
			}

			newClosest := make([]float64, n)
			var newPotential float64
			for i, x := range X {
				d := euclidean(x, X[cand])
				d *= d
				if closestSq[i] < d {
					d = closestSq[i]
				}
				newClosest[i] = d
				newPotential += d
			}
			if newPotential < bestPotential {
				bestPotential = newPotential
				bestIdx = cand
				bestClosest = newClosest
			}
		}

		centroids = append(centroids, append([]float64{}, X[bestIdx]...))
		closestSq = bestClosest
		potential = bestPotential
	}
	return centroids
}

func lloyd(X [][]float64, centroids [][]float64, maxIter int, rng *rand.Rand) (labels []int, finalCentroids [][]float64, inertia float64) {
	n := len(X)
	dim := len(X[0])
	k := len(centroids)
	labels = make([]int, n)

	for iter := 0; iter < maxIter; iter++ {
		changed := false
		for i, x := range X {
			c, _ := nearestCentroid(x, centroids)
			if c != labels[i] {
				changed = true
				labels[i] = c
			}
		}
		if iter > 0 && !changed {
			break
		}

		newCentroids := make([][]float64, k)
		counts := make([]int, k)
		for i := range newCentroids {
			newCentroids[i] = make([]float64, dim)
		}
		for i, x := range X {
			c := labels[i]
			counts[c]++
			for d := 0; d < dim; d++ {
				newCentroids[c][d] += x[d]
			}
		}
		for c := 0; c < k; c++ {
			if counts[c] == 0 {
				// re-seed an empty cluster at a random point to avoid NaNs
				newCentroids[c] = append([]float64{}, X[rng.Intn(n)]...)
				continue
			}
			for d := 0; d < dim; d++ {
				newCentroids[c][d] /= float64(counts[c])
			}
		}
		centroids = newCentroids
	}

	for i, x := range X {
		c, d := nearestCentroid(x, centroids)
		labels[i] = c
		inertia += d * d
	}
	return labels, centroids, inertia
}

// KMeans runs k-means with nInit random k-means++ restarts, keeping the
// lowest-inertia result (matching sklearn.cluster.KMeans(n_init=...)).
func KMeans(X [][]float64, k, nInit, maxIter int, seed int64) (labels []int, centroids [][]float64) {
	rng := rand.New(rand.NewSource(seed))
	bestInertia := math.Inf(1)
	for i := 0; i < nInit; i++ {
		init := kmeansPlusPlusInit(X, k, rng)
		l, c, inertia := lloyd(X, init, maxIter, rng)
		if inertia < bestInertia {
			bestInertia = inertia
			labels, centroids = l, c
		}
	}
	return labels, centroids
}

// SilhouetteScore computes the mean silhouette coefficient over all points
// (O(n^2), fine at the scale of a single engagement's host count).
func SilhouetteScore(X [][]float64, labels []int) float64 {
	n := len(X)
	clusterOf := labels
	members := map[int][]int{}
	for i, c := range clusterOf {
		members[c] = append(members[c], i)
	}
	if len(members) < 2 {
		return 0
	}

	var total float64
	for i := 0; i < n; i++ {
		ci := clusterOf[i]
		// a(i): mean distance to other points in the same cluster
		var aSum float64
		aCount := 0
		for _, j := range members[ci] {
			if j == i {
				continue
			}
			aSum += euclidean(X[i], X[j])
			aCount++
		}
		if aCount == 0 {
			// singleton cluster: sklearn's convention (and the one used
			// throughout this tool's validation against it) scores this
			// point's silhouette as exactly 0, it does NOT exclude it
			// from the mean -- excluding it would bias the score upward
			// for any k that creates lots of singletons, which is
			// exactly the degenerate case k-selection most needs to be
			// penalized for, not rewarded.
			total += 0
			continue
		}
		a := aSum / float64(aCount)

		// b(i): lowest mean distance to any other cluster
		b := math.Inf(1)
		for c, idxs := range members {
			if c == ci {
				continue
			}
			var bSum float64
			for _, j := range idxs {
				bSum += euclidean(X[i], X[j])
			}
			mean := bSum / float64(len(idxs))
			if mean < b {
				b = mean
			}
		}

		s := 0.0
		if m := math.Max(a, b); m > 0 {
			s = (b - a) / m
		}
		total += s
	}
	return total / float64(n)
}

// ChooseK runs KMeans for every k in [kMin, kMax] (step kStep) and returns
// the labels for whichever k scored highest by silhouette -- this is the
// "Method A" grid search validated against real data as at least as good
// as a cosine/normalized variant or HDBSCAN, so it's what's used by
// default rather than something more elaborate.
func ChooseK(X [][]float64, kMin, kMax, kStep, nInit, maxIter int, seed int64) (bestK int, bestLabels []int, bestSilhouette float64) {
	bestSilhouette = math.Inf(-1)
	for k := kMin; k <= kMax; k += kStep {
		if k >= len(X) {
			break
		}
		labels, _ := KMeans(X, k, nInit, maxIter, seed)
		sil := SilhouetteScore(X, labels)
		if sil > bestSilhouette {
			bestSilhouette = sil
			bestK = k
			bestLabels = labels
		}
	}
	return bestK, bestLabels, bestSilhouette
}
