package autogroup

import (
	"reflect"
	"testing"

	"realm.pub/tavern/internal/autogroup/features"
)

func TestDefaultSkip(t *testing.T) {
	if got, want := DefaultSkip(false), (map[string]bool{"extipoctet": true, "extipfull": true}); !reflect.DeepEqual(got, want) {
		t.Errorf("DefaultSkip(false) = %v, want %v", got, want)
	}
	if got, want := DefaultSkip(true), (map[string]bool{"ipoctet": true, "extipfull": true}); !reflect.DeepEqual(got, want) {
		t.Errorf("DefaultSkip(true) = %v, want %v", got, want)
	}
}

// Hosts sharing an internal IP but behind different external IPs are
// indistinguishable by default, and separable with ext=true.
func TestVectorizeExtSeparatesRemappedInternalIPs(t *testing.T) {
	mk := func(ext string) features.Doc {
		return features.BuildDoc(features.Host{ID: ext, PrimaryIP: "10.0.0.5", ExternalIP: ext, Platform: "PLATFORM_LINUX"}, nil)
	}
	docs := []features.Doc{mk("99.9.1.10"), mk("99.9.1.10"), mk("99.9.2.20"), mk("99.9.2.20")}

	def := Vectorize(docs, DefaultSkip(false))
	if !reflect.DeepEqual(def[0], def[2]) {
		t.Error("default config should see identical internal IPs as identical hosts")
	}
	ext := Vectorize(docs, DefaultSkip(true))
	if reflect.DeepEqual(ext[0], ext[2]) {
		t.Error("ext should separate hosts that differ only by external IP")
	}
	if !reflect.DeepEqual(ext[0], ext[1]) {
		t.Error("ext should keep hosts with the same external IP together")
	}
}

func TestVectorizeAllBlocksSkippedReturnsNil(t *testing.T) {
	docs := []features.Doc{features.BuildDoc(features.Host{ID: "a"}, nil)}
	all := map[string]bool{"procpath": true, "principal": true, "hostname": true, "ipoctet": true, "extipoctet": true, "extipfull": true, "platform": true}
	if got := Vectorize(docs, all); got != nil {
		t.Errorf("expected nil, got %v", got)
	}
}
