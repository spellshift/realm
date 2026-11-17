package builder

import (
	"testing"

	"realm.pub/tavern/internal/c2/c2pb"
)

func TestTransportTypeToStringMatchesImixConfig(t *testing.T) {
	if got := TransportTypeToString(c2pb.Transport_TRANSPORT_HTTP1); got != "http1" {
		t.Fatalf("HTTP1 transport must serialize as http1 for Imix, got %q", got)
	}
	for _, tc := range []struct {
		transport c2pb.Transport_Type
		want      string
	}{
		{c2pb.Transport_TRANSPORT_QUIC, "quic"},
		{c2pb.Transport_TRANSPORT_TCP_BIND, "tcp_bind"},
	} {
		if got := TransportTypeToString(tc.transport); got != tc.want {
			t.Fatalf("transport %s must serialize as %s for Imix, got %q", tc.transport, tc.want, got)
		}
	}
}
