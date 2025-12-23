package http1

import (
	"context"
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
func (r *Redirector) Redirect(ctx context.Context, listenOn string, upstream *grpc.ClientConn) error {
	mux := http.NewServeMux()
	mux.HandleFunc("/c2.C2/FetchAsset", func(w http.ResponseWriter, r *http.Request) {
		handleFetchAssetStreaming(w, r, upstream)
	})
	mux.HandleFunc("/c2.C2/ReportFile", func(w http.ResponseWriter, r *http.Request) {
		handleReportFileStreaming(w, r, upstream)
	})
	mux.HandleFunc("/c2.C2/", func(w http.ResponseWriter, r *http.Request) {
		handleHTTPRequest(w, r, upstream)
	})
	mux.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		facadeHTTPRequest(w, r, upstream)
	})

	srv := &http.Server{
		Addr:    listenOn,
		Handler: mux,
	}

	slog.Info("HTTP/1.1 redirector started", "listen_on", listenOn)
	return srv.ListenAndServe()
}

func facadeHTTPRequest(w http.ResponseWriter, r *http.Request, upstream *grpc.ClientConn) {
	w.Header().Set("Server", "nginx/1.29.4")
	if r.URL.Path == "/" {
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`<!DOCTYPE html>
<html>
<head>
<title>Welcome to nginx!</title>
<style>
html { color-scheme: light dark; }
body { width: 35em; margin: 0 auto;
font-family: Tahoma, Verdana, Arial, sans-serif; }
</style>
</head>
<body>
<h1>Welcome to nginx!</h1>
<p>If you see this page, the nginx web server is successfully installed and
working. Further configuration is required.</p>

<p>For online documentation and support please refer to
<a href="http://nginx.org/">nginx.org</a>.<br/>
Commercial support is available at
<a href="http://nginx.com/">nginx.com</a>.</p>

<p><em>Thank you for using nginx.</em></p>
</body>
</html>
`))
	} else {
		w.WriteHeader(http.StatusNotFound)
		w.Write([]byte(`<html>
<head><title>404 Not Found</title></head>
<body>
<center><h1>404 Not Found</h1></center>
<hr><center>nginx/1.29.4</center>
</body>
</html>
`))
	}

}
