package main

import (
	"context"
	"fmt"
	"io"
	"log"
	"log/slog"
	"net/http"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/encoding"
)

// RawCodec passes through raw bytes without marshaling/unmarshaling
type RawCodec struct{}

func (RawCodec) Marshal(v interface{}) ([]byte, error) {
	if b, ok := v.([]byte); ok {
		return b, nil
	}
	return nil, fmt.Errorf("failed to marshal, message is %T", v)
}

func (RawCodec) Unmarshal(data []byte, v interface{}) error {
	if b, ok := v.(*[]byte); ok {
		*b = data
		return nil
	}
	return fmt.Errorf("failed to unmarshal, message is %T", v)
}

func (RawCodec) Name() string {
	return "raw"
}

func init() {
	encoding.RegisterCodec(RawCodec{})
}

func httpRedirectorRun(ctx context.Context, upstream string, options ...func(*Config)) error {
	// Initialize Config
	cfg := &Config{}
	for _, opt := range options {
		opt(cfg)
	}

	conn, err := grpc.NewClient(
		upstream,
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	if err != nil {
		log.Fatalf("Failed to connect to gRPC server: %v", err)
	}
	defer conn.Close()

	mux := http.NewServeMux()
	mux.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		handleHTTPRequest(w, r, conn)
	})

	server := &http.Server{
		Addr:    cfg.srv.Addr,
		Handler: mux,
	}


	fmt.Printf("HTTP/1.1 proxy listening on %s, forwarding to gRPC server at %s\n", server.Addr, upstream)
	if err := server.ListenAndServe(); err != nil {
		log.Fatalf("Failed to start HTTP server: %v", err)
	}

	return nil
}

func handleHTTPRequest(w http.ResponseWriter, r *http.Request, conn *grpc.ClientConn) {
	// Only accept POST requests
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}
	println("here")

	// Extract method name from path (e.g., "/SayHello")
	methodName := r.URL.Path

	if methodName == "" {
		http.Error(w, "Method name required in path", http.StatusBadRequest)
		return
	}

	// Read the raw protobuf request body
	requestBody, err := io.ReadAll(r.Body)
	if err != nil {
		http.Error(w, fmt.Sprintf("Failed to read request body: %v", err), http.StatusBadRequest)
		return
	}
	defer r.Body.Close()

	fmt.Printf("[HTTP -> gRPC] Method: %s, Body size: %d bytes\n", methodName, len(requestBody))

	// Make gRPC call with raw bytes
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	slog.Info(fmt.Sprintf("requestBody: % 02x", requestBody))
	var responseBody []byte
	err = conn.Invoke(
		ctx,
		methodName,
		requestBody,
		&responseBody,
		grpc.CallContentSubtype("raw"),
	)

	if err != nil {
		fmt.Printf("[gRPC Error] %v\n", err)
		http.Error(w, fmt.Sprintf("gRPC call failed: %v", err), http.StatusInternalServerError)
		return
	}

	fmt.Printf("[gRPC -> HTTP] Response size: %d bytes\n", len(responseBody))

	// Write the raw protobuf response back
	w.Header().Set("Content-Type", "application/grpc")
	w.WriteHeader(http.StatusOK)
	if _, err := w.Write(responseBody); err != nil {
		fmt.Printf("[HTTP Write Error] %v\n", err)
	}
}
