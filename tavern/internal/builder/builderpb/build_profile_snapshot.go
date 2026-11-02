package builderpb

// BuildProfileSnapshot is the build recipe captured when a task is created.
// Keep this independent of Ent entities so persisted snapshots have a stable shape.
type BuildProfileSnapshot struct {
	Name            string                  `json:"name"`
	BuildImage      string                  `json:"buildImage"`
	Setupscript     string                  `json:"setupscript"`
	Prebuildscript  string                  `json:"prebuildscript"`
	BuildScript     string                  `json:"buildScript"`
	Postbuildscript string                  `json:"postbuildscript"`
	ArtifactPath    string                  `json:"artifactPath"`
	Unique          string                  `json:"unique"`
	Transports      []BuildProfileTransport `json:"transports"`
	Tomes           []BuildTomeSnapshot     `json:"tomes"`
}

// BuildTomeSnapshot preserves a tome's identity and parameters alongside its bundle.
type BuildTomeSnapshot struct {
	TomeID int    `json:"tomeID"`
	Name   string `json:"name"`
	Params string `json:"params"`
}
