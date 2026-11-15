package builder

import (
	"testing"

	"realm.pub/tavern/internal/c2/c2pb"
)

func TestTransportTypeToStringMatchesImixConfig(t *testing.T) {
	if got := TransportTypeToString(c2pb.Transport_TRANSPORT_HTTP1); got != "http1" {
		t.Fatalf("HTTP1 transport must serialize as http1 for Imix, got %q", got)
	}
}
