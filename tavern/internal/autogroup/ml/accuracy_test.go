package ml

import "testing"

func TestClusteringAccuracyPerfectMatch(t *testing.T) {
	trueLabels := []string{"a", "a", "b", "b", "c"}
	predClusters := []int{10, 10, 20, 20, 30} // different ids, same grouping
	acc := ClusteringAccuracy(trueLabels, predClusters)
	if acc != 1.0 {
		t.Errorf("expected perfect 1.0 accuracy for an exact grouping (relabeled), got %v", acc)
	}
}

func TestClusteringAccuracyWorstCase(t *testing.T) {
	// one giant cluster containing 3 different true labels equally ->
	// best possible renaming can only capture the majority class
	trueLabels := []string{"a", "a", "b", "c"}
	predClusters := []int{1, 1, 1, 1}
	acc := ClusteringAccuracy(trueLabels, predClusters)
	if acc != 0.5 { // best single rename = "a" for cluster 1, matches 2/4
		t.Errorf("expected 0.5 (majority class under one cluster), got %v", acc)
	}
}

func TestClusteringAccuracyMoreClustersThanLabels(t *testing.T) {
	// true structure has 2 labels, clustering over-split into 4 clusters
	// -- optimal assignment should still recover 1.0 since every cluster
	// is pure (each maps cleanly to one true label, even if two clusters
	// share the same true label and therefore can't both be "assigned"
	// under a strict one-to-one mapping)
	trueLabels := []string{"a", "a", "a", "b", "b"}
	predClusters := []int{1, 1, 2, 3, 3}
	acc := ClusteringAccuracy(trueLabels, predClusters)
	// optimal: cluster1->a (2/2), cluster3->b (2/2), cluster2 unmatched (1 wrong)
	want := 4.0 / 5.0
	if acc != want {
		t.Errorf("expected %v, got %v", want, acc)
	}
}

func TestClusteringAccuracyMisleadingClusterIDsDontMatter(t *testing.T) {
	// cluster ids are arbitrary ints with no relation to label ordering;
	// make sure the algorithm doesn't accidentally rely on id ordering
	trueLabels := []string{"x", "y", "x", "y"}
	predClusters := []int{-5, 999, -5, 999}
	acc := ClusteringAccuracy(trueLabels, predClusters)
	if acc != 1.0 {
		t.Errorf("expected 1.0 regardless of arbitrary cluster id values, got %v", acc)
	}
}

func TestHungarianKnownOptimalAssignment(t *testing.T) {
	// classic 3x3 minimum assignment example with a known optimal cost
	cost := [][]float64{
		{4, 2, 8},
		{4, 3, 7},
		{3, 1, 6},
	}
	assignment := hungarian(cost)
	var total float64
	seenCols := map[int]bool{}
	for i, j := range assignment {
		if j < 0 || seenCols[j] {
			t.Fatalf("invalid assignment: row %d -> col %d (seen=%v)", i, j, seenCols)
		}
		seenCols[j] = true
		total += cost[i][j]
	}
	// optimal here: (0,1)=2 + (1,2)=7 + (2,0)=3 = 12, or (0,1)=2+(1,0)=4+(2,2)=6=12;
	// brute force over all 6 permutations confirms 12 is the minimum
	if total != 12 {
		t.Errorf("expected minimum cost 12, got %v (assignment=%v)", total, assignment)
	}
}
