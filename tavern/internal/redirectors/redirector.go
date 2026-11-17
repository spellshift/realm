package redirectors

import (
	"context"
	"crypto/tls"
	"fmt"
	"maps"
	"slices"
	"sync"

	"google.golang.org/grpc"
	"realm.pub/tavern/internal/logging"
)

var (
	log         = logging.For("redirectors")
	mu          sync.RWMutex
	redirectors = make(map[string]Redirector)
)

// A Redirector for traffic to an upstream gRPC server
type Redirector interface {
	Redirect(ctx context.Context, listenOn string, upstream *grpc.ClientConn, tlsConfig *tls.Config) error
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

// Run starts the redirector with the given name, connecting to the specified upstream address.
// If tlsConfig is non-nil, the redirector will use TLS for incoming connections.
func Run(ctx context.Context, name string, listenOn string, upstreamAddr string, tlsConfig *tls.Config) error {
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
		log.DebugContext(ctx, "closing connection to upstream gRPC", "redirector_name", name)
		upstream.Close()
	}()
	log.DebugContext(ctx, "connected to upstream gRPC", "redirector_name", name, "upstream_addr", upstreamAddr)

	// Start the redirector
	return redirector.Redirect(ctx, listenOn, upstream, tlsConfig)
}
