package executor

import (
	"context"
)

// BuildSpec contains the parameters for a build task execution.
type BuildSpec struct {
	TaskID      int64
	TargetOS    string
	BuildImage  string
	BuildScript string
}

// Executor defines the interface for build task execution.
// Implementations must stream output and errors over the provided channels
// as the build progresses. Both channels are closed by the caller after
// Build returns.
type Executor interface {
	// Build executes a build task described by spec. As the build runs,
	// stdout lines are sent to outputCh and stderr lines to errorCh.
	// Build blocks until the build completes (or the context is cancelled)
	// and returns any execution error.
	Build(ctx context.Context, spec BuildSpec, outputCh chan<- string, errorCh chan<- string) error
}
