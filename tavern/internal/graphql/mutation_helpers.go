package graphql

import (
	"strings"

	"realm.pub/tavern/internal/builder/builderpb"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/graphql/models"
)

// normalizeUpstream ensures the upstream address has a scheme so that
// url.Parse correctly identifies the host and port. Bare "host:port" values
// default to https.
func normalizeUpstream(raw string) string {
	if raw == "" {
		return raw
	}

	// Already has a scheme – nothing to do.
	if strings.Contains(raw, "://") {
		return raw
	}

	// No scheme: default to https so the client-side url.Parse works correctly.
	return "https://" + raw
}

// convertTransportInputs converts GraphQL transport inputs to the builderpb type.
func convertTransportInputs(inputs []*models.BuildTaskTransportInput) []builderpb.BuildTaskTransport {
	transports := make([]builderpb.BuildTaskTransport, len(inputs))
	for i, t := range inputs {
		var extra string
		if t.Extra != nil {
			extra = *t.Extra
		}
		transports[i] = builderpb.BuildTaskTransport{
			URI:      t.URI,
			Interval: t.Interval,
			Type:     c2pb.Transport_Type(t.Type),
			Extra:    extra,
		}
	}
	return transports
}
