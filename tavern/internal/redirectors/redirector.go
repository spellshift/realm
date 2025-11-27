package redirectors

import (
	"context"
	"fmt"
	"log/slog"
	"maps"
	"slices"
	"sync"

	"github.com/urfave/cli"
	"google.golang.org/grpc"
)

var (
	mu          sync.RWMutex
	redirectors = make(map[string]Redirector)
)

// A Redirector for traffic to an upstream gRPC server
type Redirector interface {
	Redirect(ctx context.Context, listenOn string, upstream *grpc.ClientConn, opts map[string]interface{}) error
}

// FlagProvider is an optional interface that redirectors can implement to provide custom CLI flags
type FlagProvider interface {
	Flags() []cli.Flag
}

// Register makes a Redirector available by the provided name.
// If Register is called twice with the same name or if driver is nil,it panics.
func Register(name string, redirector Redirector) {
	mu.Lock()
	defer mu.Unlock()
	if redirector == nil {
		panic("redirectors: Register redirector is nil")
	}
	if _, dup := redirectors[name]; dup {
		panic("redirectors: Register called twice for redirector " + name)
	}
	redirectors[name] = redirector
}

// List returns a sorted list of the names of the registered redirectors.
func List() []string {
	mu.RLock()
	defer mu.RUnlock()
	return slices.Sorted(maps.Keys(redirectors))
}

// GetAll returns all registered redirectors (for flag collection)
func GetAll() map[string]Redirector {
	mu.RLock()
	defer mu.RUnlock()
	result := make(map[string]Redirector, len(redirectors))
	for k, v := range redirectors {
		result[k] = v
	}
	return result
}

// Run starts the redirector with the given name, connecting to the specified upstream address
func Run(ctx context.Context, name string, listenOn string, upstreamAddr string, opts map[string]interface{}) error {
	// Get the Redirector
	mu.RLock()
	redirector, exists := redirectors[name]
	mu.RUnlock()
	if !exists || redirector == nil {
		return fmt.Errorf("redirector %q not found", name)
	}

	// Connect to the upstream gRPC server
	upstream, err := ConnectToUpstream(upstreamAddr)
	if err != nil {
		return fmt.Errorf("failed to connect to upstream: %v", err)
	}
	defer func() {
		slog.DebugContext(ctx, "redirectors: closing connection to upstream grpc", "redirector_name", name)
		upstream.Close()
	}()
	slog.DebugContext(ctx, "redirectors: connected to upstream grpc", "redirector_name", name, "upstream_addr", upstreamAddr)

	// Start the redirector
	return redirector.Redirect(ctx, listenOn, upstream, opts)
}
