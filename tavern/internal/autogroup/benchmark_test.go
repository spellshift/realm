package autogroup

import (
	"os"
	"testing"

	"realm.pub/tavern/internal/autogroup/benchmark"
	"realm.pub/tavern/internal/autogroup/features"
	"realm.pub/tavern/internal/autogroup/ml"
)

// defaultBenchmarkDump is used when AUTOGROUP_BENCHMARK_DUMP is unset. This
// is a multi-GB fixture that lives outside the repo (not checked in), so
// the test skips rather than fails when it's absent -- see
// TestBenchmarkAgainstSeccdcDump below.
const defaultBenchmarkDump = "/persistent/workspaces/db_backups/2026-v0.3.3-seccdc.sql"

// TestBenchmarkAgainstSeccdcDump scores autogroup's clustering against a
// past engagement's own already-applied tags, read directly from a raw
// mysqldump backup (seccdc specifically -- rmccdc's data is too
// small/poor quality to benchmark meaningfully, per project convention).
//
// It mirrors the oracle-k / auto-k comparison documented in the deleted
// bin/host-autogroup CLI's benchmark README:
//
//	dump:            db_backups/2026-v0.3.3-seccdc.sql
//	tome:            "Process list"
//	labeled hosts:   239
//	true # groups:   23
//
//	mode                             k used   accuracy      ARI silhouette
//	oracle-k (true count)                23      1.000    1.000        n/a
//	auto-k (unsupervised)                24      0.987    0.990      0.344
//
// RNGSeed is fixed at int64(0), so this run should be exactly
// reproducible; a result other than k=24/acc=0.987/ARI=0.990 for auto-k is
// worth flagging clearly rather than quietly passing a loose assertion.
func TestBenchmarkAgainstSeccdcDump(t *testing.T) {
	path := os.Getenv("AUTOGROUP_BENCHMARK_DUMP")
	if path == "" {
		path = defaultBenchmarkDump
	}
	if _, err := os.Stat(path); err != nil {
		t.Skipf("benchmark dump not available at %s (set AUTOGROUP_BENCHMARK_DUMP to override): %v", path, err)
	}

	all, err := benchmark.Load(path, "Process list")
	if err != nil {
		t.Fatalf("benchmark.Load: %v", err)
	}

	var docs []features.Doc
	var trueLabels []string
	for _, h := range all {
		if h.Label == "" {
			continue
		}
		docs = append(docs, h.Doc)
		trueLabels = append(trueLabels, h.Label)
	}
	distinctLabels := map[string]bool{}
	for _, l := range trueLabels {
		distinctLabels[l] = true
	}
	nGroups := len(distinctLabels)

	t.Logf("labeled hosts: %d, true # groups: %d", len(docs), nGroups)
	if nGroups < 2 {
		t.Fatalf("only %d distinct tag(s) found -- nothing meaningful to benchmark against", nGroups)
	}
	if len(docs) < nGroups {
		t.Fatalf("only %d labeled hosts for %d distinct tags -- too few to benchmark", len(docs), nGroups)
	}

	X := Vectorize(docs, DefaultSkip(false))

	// --- oracle-k: k pinned to the true group count ---
	oracleLabels, _ := ml.KMeans(X, nGroups, NInit, MaxIter, RNGSeed)
	oracleAcc := ml.ClusteringAccuracy(trueLabels, oracleLabels)
	oracleARI := ml.AdjustedRandIndex(trueLabels, oracleLabels)
	t.Logf("oracle-k: k=%d accuracy=%.3f ARI=%.3f", nGroups, oracleAcc, oracleARI)

	if oracleAcc < 0.95 {
		t.Errorf("oracle-k accuracy = %.3f, want >= 0.95 (baseline 1.000)", oracleAcc)
	}
	if oracleARI < 0.95 {
		t.Errorf("oracle-k ARI = %.3f, want >= 0.95 (baseline 1.000)", oracleARI)
	}

	// --- auto-k: fully unsupervised, k auto-selected via silhouette grid search ---
	autoK, autoLabels, autoSil := ChooseKAuto(X)
	autoAcc := ml.ClusteringAccuracy(trueLabels, autoLabels)
	autoARI := ml.AdjustedRandIndex(trueLabels, autoLabels)
	t.Logf("auto-k: k=%d accuracy=%.3f ARI=%.3f silhouette=%.3f", autoK, autoAcc, autoARI, autoSil)

	if autoAcc < 0.95 {
		t.Errorf("auto-k accuracy = %.3f, want >= 0.95 (baseline 0.987)", autoAcc)
	}
	if autoARI < 0.95 {
		t.Errorf("auto-k ARI = %.3f, want >= 0.95 (baseline 0.990)", autoARI)
	}
	if diff := autoK - nGroups; diff < -2 || diff > 2 {
		t.Errorf("auto-k chose k=%d, true count is %d (diff %d), want within 2 (baseline k=24 vs true=23)", autoK, nGroups, diff)
	}

	// RNGSeed is fixed, so this exact run should be fully reproducible --
	// flag loudly (without failing the build) if it drifts from the
	// documented baseline, since that would mean something upstream of
	// this test changed the ML pipeline's behavior.
	if autoK != 24 || round3(autoAcc) != 0.987 || round3(autoARI) != 0.990 {
		t.Logf("NOTE: auto-k result (k=%d, accuracy=%.3f, ARI=%.3f) differs from the exact documented baseline (k=24, accuracy=0.987, ARI=0.990) despite a fixed RNGSeed -- investigate if this wasn't expected", autoK, autoAcc, autoARI)
	}
}

func round3(f float64) float64 {
	return float64(int(f*1000+0.5)) / 1000
}
