package redirectors

import (
	"context"
	"sync/atomic"
)

// Gate is shared by every listener in one managed redirector deployment.
// A nil gate (unmanaged redirector) always allows work.
type Gate struct{ enabled atomic.Bool }

func NewGate() *Gate {
	g := &Gate{}
	g.enabled.Store(true)
	return g
}

func (g *Gate) SetEnabled(enabled bool) { g.enabled.Store(enabled) }

func (g *Gate) Enabled() bool { return g == nil || g.enabled.Load() }

type gateContextKey struct{}

func WithGate(ctx context.Context, gate *Gate) context.Context {
	return context.WithValue(ctx, gateContextKey{}, gate)
}

func Allowed(ctx context.Context) bool {
	gate, _ := ctx.Value(gateContextKey{}).(*Gate)
	return gate.Enabled()
}
