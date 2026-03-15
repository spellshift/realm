package builderpb

import "realm.pub/tavern/internal/c2/c2pb"

// BuildProfileTransport represents a single transport configuration stored in the BuildTask entity.
type BuildProfileTransport struct {
	URI           string              `json:"uri"`
	Interval      int                 `json:"interval"`
	Type          c2pb.Transport_Type `json:"type"`
	Extra         string              `json:"extra,omitempty"`
}
