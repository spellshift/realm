package ml

import "math"

import "testing"

func approxEqual(a, b, tol float64) bool { return math.Abs(a-b) <= tol }

func TestAdjustedRandIndexPerfectMatch(t *testing.T) {
	trueLabels := []string{"a", "a", "b", "b", "c", "c"}
	pred := []int{1, 1, 2, 2, 3, 3}
	ari := AdjustedRandIndex(trueLabels, pred)
	if !approxEqual(ari, 1.0, 1e-9) {
		t.Errorf("expected ARI=1.0 for a perfect relabeled match, got %v", ari)
	}
}

func TestAdjustedRandIndexKnownValue(t *testing.T) {
	// classic textbook example (see sklearn's own doctest for
	// adjusted_rand_score): labels_true=[0,0,0,1,1,1], labels_pred=[0,0,1,1,2,2]
	// known ARI = 0.2424242424242424
	trueLabels := []string{"0", "0", "0", "1", "1", "1"}
	pred := []int{0, 0, 1, 1, 2, 2}
	ari := AdjustedRandIndex(trueLabels, pred)
	want := 0.2424242424242424
	if !approxEqual(ari, want, 1e-6) {
		t.Errorf("expected ARI=%v (sklearn reference value), got %v", want, ari)
	}
}

func TestAdjustedRandIndexSingleClusterEachSide(t *testing.T) {
	trueLabels := []string{"a", "a", "a"}
	pred := []int{1, 1, 1}
	ari := AdjustedRandIndex(trueLabels, pred)
	if ari != 1 {
		t.Errorf("expected ARI=1 for trivial single-cluster-both-sides case, got %v", ari)
	}
}
