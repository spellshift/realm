package http1

import (
	"context"
	"crypto/tls"
	"log/slog"
	"net/http"

	"google.golang.org/grpc"
	"realm.pub/tavern/internal/redirectors"
)

func init() {
	redirectors.Register("http1", &Redirector{})
}

// A Redirector implementation which receives HTTP/1.1 traffic locally and
// sends gRPC traffic to the upstream destination.
type Redirector struct{}

// Redirect starts the redirector, listening for traffic locally and forwarding to the upstream
func (r *Redirector) Redirect(ctx context.Context, listenOn string, upstream *grpc.ClientConn, tlsConfig *tls.Config) error {
	mux := http.NewServeMux()
	mux.HandleFunc("/c2.C2/FetchAsset", func(w http.ResponseWriter, r *http.Request) {
		handleFetchAssetStreaming(w, r, upstream)
	})
	mux.HandleFunc("/c2.C2/ReportFile", func(w http.ResponseWriter, r *http.Request) {
		handleReportFileStreaming(w, r, upstream)
	})
	mux.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		handleHTTPRequest(w, r, upstream)
	})

	srv := &http.Server{
		Addr:      listenOn,
		Handler:   mux,
		TLSConfig: tlsConfig,
	}

	if tlsConfig != nil {
		slog.Debug("http1 redirector: TLS enabled", "listen_on", listenOn, "min_version", tlsConfig.MinVersion, "num_certificates", len(tlsConfig.Certificates))
		slog.Info("http1 redirector: HTTPS started", "listen_on", listenOn)
		return srv.ListenAndServeTLS("", "")
	}

	slog.Info("http1 redirector: HTTP started", "listen_on", listenOn)
	return srv.ListenAndServe()
}
