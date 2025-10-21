package transport

import (
	"context"
	"fmt"
	"log"
	"net/http"
	"os"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

type HTTP1Transport struct {
	client   *grpc.ClientConn
	server   *http.Server
	upstream string
	logger *log.Logger
}

func NewHTTP1Transport(server *http.Server, upstream string) (*HTTP1Transport, error) {
	client, err := grpc.NewClient(upstream,
		grpc.WithTransportCredentials(insecure.NewCredentials()))

	if err != nil {
		return nil, err
	}
	return &HTTP1Transport{
		client:   client,
		server:   server,
		upstream: upstream,
		logger: log.New(os.Stderr, "[HTTP1 Transport] ", log.Flags()),
	}, nil
}

func (t *HTTP1Transport) ForwardRequests(ctx context.Context) error {
	// Configure Request Logging
	// Implement the logic to forward HTTP/1.1 requests here
	fmt.Printf("forwarding HTTP/1.1 requests from %s to upstream Tavern server: %s\n", t.server.Addr, t.upstream)
	http.ListenAndServe(t.server.Addr, t)

	return nil
}

type Client struct {
	client *http.Client
}

// ServeHTTP implements http.Handler.
func (t HTTP1Transport) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	t.logger.Printf("Received request: %s %s", r.Method, r.URL.Path)
    conn, err := grpc.Dial("127.0.0.1:80", grpc.WithInsecure())
    if err != nil {
        log.Fatalf("fail to dial: %v", err)
    }
    defer conn.Close()

	t.logger.Printf("request bytes: %v\n", r.Body)
}

/*
export SECRETS_FILE_PATH='/tmp/secrets'; go run ./tavern/

export HTTP_LISTEN_ADDR="0.0.0.0:8080"; export SECRETS_FILE_PATH="/tmp/secrets"; go run ./tavern/ transport http1 --upstream 127.0.0.1:80

export IMIX_SERVER_PUBKEY="GUdN7c6/4qRlTvxazCsryioupTI4VcHRYR+ST8YA7Qc="; export IMIX_CALLBACK_URI="http://127.0.0.1:8080"; cargo run --bin imix

*/
