package main

import (
	"flag"
	"log"
	"os"

	"realm.pub/tavern/portals/portalpb"
)

const (
	authCachePath = ".tavern-auth"
)

func main() {
	if len(os.Args) > 1 && os.Args[1] == "trace" {
		traceCommand(os.Args[2:])
		return
	}

	proxyCommand()
}

func proxyCommand() {
	// Re-parse flags for proxy command since flag.Parse() uses os.Args by default
	// and we might have skipped "trace".
	// But since we checked os.Args[1] manually, standard flag.Parse() will fail if we don't adjust os.Args
	// or create a new FlagSet.
	// Since the original code used the default flag set, let's stick to that but we need to ensure
	// trace command doesn't interfere.

	// Actually, best to just use a custom FlagSet for proxy too, or reset os.Args?
	// The original code used global flags.

	// Let's manually define the proxy flags again using a FlagSet to avoid conflict with `trace` flags if we were to define them globally.
	fs := flag.NewFlagSet("proxy", flag.ExitOnError)
	portalID := fs.Int64("portal", 0, "Portal ID")
	listenAddr := fs.String("listen", "127.0.0.1:1080", "SOCKS5 Listen Address")
	upstreamAddr := fs.String("upstream", "127.0.0.1:8000", "Upstream gRPC Address")

	// Parse remaining args
	// If main was called without subcommands, os.Args is [prog, flags...]
	// If called with `trace`, we handled it.
	// So just parse os.Args[1:]
	fs.Parse(os.Args[1:])

	if *portalID == 0 {
		log.Fatal("portal is required")
	}

	p := &Proxy{
		portalID:     *portalID,
		listenAddr:   *listenAddr,
		upstreamAddr: *upstreamAddr,
		streams:      make(map[string]chan *portalpb.Mote),
	}

	if err := p.Run(); err != nil {
		log.Fatalf("Proxy failed: %v", err)
	}
}
