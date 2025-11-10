package main

import (
	"context"
	"fmt"
	"io"
	"log"
	"net/http"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

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

	server := cfg.srv

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

	// Extract method name from path (e.g., "/SayHello")
	methodName := r.URL.Path
	if len(methodName) > 0 && methodName[0] == '/' {
		methodName = methodName[1:]
	}

	if methodName == "" {
		http.Error(w, "Method name required in path", http.StatusBadRequest)
		return
	}

	// Construct full gRPC method path
	fullMethod := fmt.Sprintf("/%s/%s", "c2.C2", methodName)

	// Read the raw protobuf request body
	requestBody, err := io.ReadAll(r.Body)
	if err != nil {
		http.Error(w, fmt.Sprintf("Failed to read request body: %v", err), http.StatusBadRequest)
		return
	}
	defer r.Body.Close()

	fmt.Printf("[HTTP -> gRPC] Method: %s, Body size: %d bytes\n", fullMethod, len(requestBody))

	// Make gRPC call with raw bytes
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	var responseBody []byte
	err = conn.Invoke(
		ctx,
		fullMethod,
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
