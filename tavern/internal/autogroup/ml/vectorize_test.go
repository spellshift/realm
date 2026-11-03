package ml

import "testing"

func TestTFIDFVectorizerDropsRareTermsAndCapsVocab(t *testing.T) {
	docs := []map[string]int{
		{"common": 2, "rare": 1},
		{"common": 1, "also_common": 3},
		{"common": 1, "also_common": 1},
	}
	v := &TFIDFVectorizer{MinDF: 2, MaxFeatures: 10}
	v.Fit(docs)

	// "rare" appears in only 1 doc, should be dropped (minDF=2)
	for _, term := range v.vocab {
		if term == "rare" {
			t.Errorf("expected 'rare' to be dropped (df=1 < minDF=2), vocab=%v", v.vocab)
		}
	}
	if v.Dim() != 2 {
		t.Errorf("expected 2 surviving terms (common, also_common), got %d: %v", v.Dim(), v.vocab)
	}
}

func TestTFIDFTransformIsL2Normalized(t *testing.T) {
	docs := []map[string]int{
		{"a": 1, "b": 1},
		{"a": 1, "b": 1},
	}
	v := &TFIDFVectorizer{MinDF: 1, MaxFeatures: 0}
	v.Fit(docs)
	out := v.Transform(map[string]int{"a": 3, "b": 1})

	var sumSq float64
	for _, x := range out {
		sumSq += x * x
	}
	if diff := sumSq - 1.0; diff > 1e-9 || diff < -1e-9 {
		t.Errorf("expected unit L2 norm (sum of squares = 1), got %v", sumSq)
	}
}

func TestBinaryVectorizerIsUnweightedPresence(t *testing.T) {
	docs := []map[string]int{
		{"x": 5},
		{"x": 1, "y": 1},
	}
	v := &BinaryVectorizer{MinDF: 1}
	v.Fit(docs)
	out := v.Transform(map[string]int{"x": 100})
	for i, val := range out {
		if val != 0 && val != 1 {
			t.Errorf("binary vectorizer should only emit 0/1, got %v at index %d", val, i)
		}
	}
}

func TestOneHotVectorizerUnknownValueIsAllZero(t *testing.T) {
	v := &OneHotVectorizer{}
	v.Fit([]string{"linux", "windows"})
	out := v.Transform("macos") // unseen at fit time
	for _, x := range out {
		if x != 0 {
			t.Errorf("expected all-zero vector for unseen category, got %v", out)
		}
	}
	seen := v.Transform("linux")
	var sum float64
	for _, x := range seen {
		sum += x
	}
	if sum != 1 {
		t.Errorf("expected exactly one hot element for a known category, got sum=%v in %v", sum, seen)
	}
}
