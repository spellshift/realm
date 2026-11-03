package ml

import "testing"

// TestKMeansSeparatesObviousClusters checks k-means recovers two
// trivially well-separated blobs.
func TestKMeansSeparatesObviousClusters(t *testing.T) {
	X := [][]float64{
		{0, 0}, {0, 1}, {1, 0}, {1, 1}, // cluster A, near origin
		{100, 100}, {100, 101}, {101, 100}, {101, 101}, // cluster B, far away
	}
	labels, _ := KMeans(X, 2, 5, 50, 0)

	for i := 1; i < 4; i++ {
		if labels[i] != labels[0] {
			t.Errorf("point %d should share cluster A's label %d, got %d", i, labels[0], labels[i])
		}
	}
	for i := 5; i < 8; i++ {
		if labels[i] != labels[4] {
			t.Errorf("point %d should share cluster B's label %d, got %d", i, labels[4], labels[i])
		}
	}
	if labels[0] == labels[4] {
		t.Errorf("clusters A and B should not share a label")
	}
}

func TestSilhouetteScorePrefersBetterK(t *testing.T) {
	X := [][]float64{
		{0, 0}, {0, 1}, {1, 0}, {1, 1},
		{100, 100}, {100, 101}, {101, 100}, {101, 101},
	}
	goodLabels, _ := KMeans(X, 2, 5, 50, 0)
	goodSil := SilhouetteScore(X, goodLabels)

	// force a bad k=4 split by labeling pairs arbitrarily within each true blob
	badLabels := []int{0, 1, 0, 1, 2, 3, 2, 3}
	badSil := SilhouetteScore(X, badLabels)

	if goodSil <= badSil {
		t.Errorf("expected k=2 (true structure) to score higher silhouette than a spurious k=4 split: good=%.3f bad=%.3f", goodSil, badSil)
	}
}

func TestChooseKRecoversTrueClusterCount(t *testing.T) {
	// three obviously separated blobs
	X := [][]float64{
		{0, 0}, {0, 1}, {1, 0},
		{50, 50}, {50, 51}, {51, 50},
		{-50, 50}, {-50, 51}, {-51, 50},
	}
	k, labels, _ := ChooseK(X, 2, 6, 1, 5, 50, 0)
	if k != 3 {
		t.Errorf("expected ChooseK to recover k=3, got k=%d", k)
	}
	if len(labels) != len(X) {
		t.Errorf("expected one label per point, got %d labels for %d points", len(labels), len(X))
	}
}

func TestEuclidean(t *testing.T) {
	d := euclidean([]float64{0, 0}, []float64{3, 4})
	if d != 5 {
		t.Errorf("expected distance 5, got %v", d)
	}
}
