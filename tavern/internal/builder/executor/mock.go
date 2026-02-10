package executor

import (
	"context"
)

// MockExecutor is a test double for the Executor interface.
// It records calls and allows tests to configure the output, errors,
// and return value for each invocation.
type MockExecutor struct {
	// BuildCalls records each BuildSpec passed to Build.
	BuildCalls []BuildSpec

	// BuildFn, if set, is called for each Build invocation.
	// It gives tests full control over what gets sent to the channels
	// and what error (if any) is returned.
	BuildFn func(ctx context.Context, spec BuildSpec, outputCh chan<- string, errorCh chan<- string) error
}

// NewMockExecutor returns a MockExecutor with no configured behavior.
// By default, Build succeeds immediately without producing output.
func NewMockExecutor() *MockExecutor {
	return &MockExecutor{}
}

// Build implements Executor. It records the call and delegates to BuildFn
// if set, otherwise returns nil.
func (m *MockExecutor) Build(ctx context.Context, spec BuildSpec, outputCh chan<- string, errorCh chan<- string) error {
	m.BuildCalls = append(m.BuildCalls, spec)
	if m.BuildFn != nil {
		return m.BuildFn(ctx, spec, outputCh, errorCh)
	}
	return nil
}
