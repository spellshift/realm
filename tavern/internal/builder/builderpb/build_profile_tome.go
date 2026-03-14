package builderpb

// BuildProfileTome represents a tome configuration stored in the BuildProfile entity.
type BuildProfileTome struct {
	TomeID int    `json:"tome_id"`
	Params string `json:"params"`
}
