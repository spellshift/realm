package builderpb

import "realm.pub/tavern/internal/c2/c2pb"

// BuildTaskTransport represents a single transport configuration stored in the BuildTask entity.
type BuildTaskTransport struct {
	URI           string              `json:"uri"`
	Interval      int                 `json:"interval"`
	Type          c2pb.Transport_Type `json:"type"`
	Extra         string              `json:"extra,omitempty"`
}
