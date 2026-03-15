package executor

import (
	"context"
)

const (
	// ExpectedExitCode is the expected container exit code for a successful build.
	ExpectedExitCode int64 = 0
)

// BuildSpec contains the parameters for a build task execution.
type BuildSpec struct {
	TaskID      int64
	TargetOS    string
	BuildImage  string
	BuildScript string

	// ArtifactPath is the path inside the container to extract after the build.
	// If empty, no artifact extraction is performed.
	ArtifactPath string

	// Env is a list of environment variables to set in the build container,
	// in the form "KEY=VALUE".
	Env []string

	// SetupScript is a shell script to run during the setup phase.
	// Written to /mnt/scripts/0_setup.sh inside the container.
	SetupScript string

	// PreBuildScript is a shell script to run before the build command.
	// Written to /mnt/scripts/1_pre_build.sh inside the container.
	PreBuildScript string

	// PostBuildScript is a shell script to run after the build command.
	// Written to /mnt/scripts/9_post_build.sh inside the container.
	PostBuildScript string

	// TomesDir is a local directory containing tome files to copy into
	// the container at /mnt/tomes.
	TomesDir string

	// Tomes contains packaged tome data downloaded via the DownloadTome RPC.
	// Each entry's Contents is a tar.gz archive that gets extracted into
	// /mnt/tomes/<ID>/ inside the container.
	Tomes []TomeData
}

// TomeData represents a single tome's packaged content downloaded from the server.
type TomeData struct {
	ID       int64
	Name     string
	Contents []byte // tar.gz archive of main.eldritch + assets
	Params   string
}

// BuildResult holds the results of a build execution, including any extracted artifacts.
type BuildResult struct {
	// ExitCode is the container's exit status code.
	ExitCode int64

	// Artifact contains the raw bytes of the extracted build artifact.
	// nil if no artifact was configured or extraction failed.
	Artifact []byte

	// ArtifactName is the filename of the extracted artifact.
	ArtifactName string
}

// Executor defines the interface for build task execution.
// Implementations must stream output and errors over the provided channels
// as the build progresses. Implementations MUST close both outputCh and
// errorCh before returning, regardless of whether an error occurred.
type Executor interface {
	// Build executes a build task described by spec. As the build runs,
	// stdout lines are sent to outputCh and stderr lines to errorCh.
	// Build blocks until the build completes (or the context is cancelled)
	// and returns a BuildResult (which may contain extracted artifacts) and
	// any execution error. Build must close both channels before returning.
	Build(ctx context.Context, spec BuildSpec, outputCh chan<- string, errorCh chan<- string) (*BuildResult, error)
}
