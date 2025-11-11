package main

import (
	"context"
	"encoding/binary"
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
	mux.HandleFunc("/c2.C2/FetchAsset", func(w http.ResponseWriter, r *http.Request) {
		handleFetchAssetStreaming(w, r, conn)
	})
	mux.HandleFunc("/c2.C2/ReportFile", func(w http.ResponseWriter, r *http.Request) {
		handleReportFileStreaming(w, r, conn)
	})
	mux.HandleFunc("/c2.C2/ReverseShell", func(w http.ResponseWriter, r *http.Request) {
		handleReverseShell(w, r, conn)
	})
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

func handleFetchAssetStreaming(w http.ResponseWriter, r *http.Request, conn *grpc.ClientConn) {
	// Only accept POST requests
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	// Read the raw protobuf request body
	requestBody, err := io.ReadAll(r.Body)
	if err != nil {
		http.Error(w, fmt.Sprintf("Failed to read request body: %v", err), http.StatusBadRequest)
		return
	}
	defer r.Body.Close()

	fmt.Printf("[HTTP -> gRPC Streaming] Method: /c2.C2/FetchAsset, Body size: %d bytes\n", len(requestBody))

	// Create a streaming call
	ctx, cancel := context.WithTimeout(context.Background(), 60*time.Second)
	defer cancel()

	stream, err := conn.NewStream(
		ctx,
		&grpc.StreamDesc{
			StreamName:    "FetchAsset",
			ServerStreams: true,
			ClientStreams: false,
		},
		"/c2.C2/FetchAsset",
		grpc.CallContentSubtype("raw"),
	)
	if err != nil {
		fmt.Printf("[gRPC Stream Error] Failed to create stream: %v\n", err)
		http.Error(w, fmt.Sprintf("Failed to create gRPC stream: %v", err), http.StatusInternalServerError)
		return
	}

	// Send the request
	if err := stream.SendMsg(requestBody); err != nil {
		fmt.Printf("[gRPC Stream Error] Failed to send request: %v\n", err)
		http.Error(w, fmt.Sprintf("Failed to send gRPC request: %v", err), http.StatusInternalServerError)
		return
	}

	// Close the send side
	if err := stream.CloseSend(); err != nil {
		fmt.Printf("[gRPC Stream Error] Failed to close send: %v\n", err)
		http.Error(w, fmt.Sprintf("Failed to close gRPC send: %v", err), http.StatusInternalServerError)
		return
	}

	// Set headers and start streaming response
	w.Header().Set("Content-Type", "application/grpc")
	w.WriteHeader(http.StatusOK)

	// Get flusher for chunked transfer encoding
	flusher, ok := w.(http.Flusher)
	if !ok {
		http.Error(w, "Streaming not supported", http.StatusInternalServerError)
		return
	}

	// Stream each encrypted chunk as it arrives from gRPC
	chunkCount := 0
	totalBytes := 0

	for {
		var responseChunk []byte
		err := stream.RecvMsg(&responseChunk)
		if err == io.EOF {
			// Stream finished successfully
			break
		}
		if err != nil {
			fmt.Printf("[gRPC Stream Error] Failed to receive message: %v\n", err)
			return
		}

		chunkCount++
		totalBytes += len(responseChunk)
		fmt.Printf("[gRPC Stream] Received chunk %d: %d bytes\n", chunkCount, len(responseChunk))

		// Write gRPC frame header: [compression_flag(1)][length(4)]
		var frameHeader [5]byte
		frameHeader[0] = 0x00 // No compression
		binary.BigEndian.PutUint32(frameHeader[1:], uint32(len(responseChunk)))

		if _, err := w.Write(frameHeader[:]); err != nil {
			fmt.Printf("[HTTP Write Error] Failed to write frame header: %v\n", err)
			return
		}

		// Write encrypted chunk immediately to HTTP client
		if _, err := w.Write(responseChunk); err != nil {
			fmt.Printf("[HTTP Write Error] Failed to write chunk: %v\n", err)
			return
		}

		// Flush to send chunk immediately (HTTP chunked transfer encoding)
		flusher.Flush()
	}

	fmt.Printf("[gRPC -> HTTP] Streamed %d chunks, total %d bytes\n", chunkCount, totalBytes)
}

func handleReportFileStreaming(w http.ResponseWriter, r *http.Request, conn *grpc.ClientConn) {
	// Only accept POST requests
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	fmt.Printf("[HTTP -> gRPC Client Streaming] Method: /c2.C2/ReportFile\n")

	// Create a client streaming call
	ctx, cancel := context.WithTimeout(context.Background(), 60*time.Second)
	defer cancel()

	stream, err := conn.NewStream(
		ctx,
		&grpc.StreamDesc{
			StreamName:    "ReportFile",
			ServerStreams: false,
			ClientStreams: true,
		},
		"/c2.C2/ReportFile",
		grpc.CallContentSubtype("raw"),
	)
	if err != nil {
		fmt.Printf("[gRPC Stream Error] Failed to create stream: %v\n", err)
		http.Error(w, fmt.Sprintf("Failed to create gRPC stream: %v", err), http.StatusInternalServerError)
		return
	}

	// Read gRPC-framed messages from HTTP request body and forward to gRPC stream
	buffer := make([]byte, 0, 64*1024) // 64KB buffer
	chunkCount := 0

	for {
		// Read HTTP body data
		readBuf := make([]byte, 32*1024) // Read 32KB at a time
		n, readErr := r.Body.Read(readBuf)
		if n > 0 {
			buffer = append(buffer, readBuf[:n]...)
		}

		// Process complete gRPC frames from buffer
		for len(buffer) >= 5 {
			// Read gRPC frame header: [compression_flag(1)][length(4)]
			compressionFlag := buffer[0]
			messageLength := binary.BigEndian.Uint32(buffer[1:5])

			// Check if we have the complete message
			if len(buffer) < 5+int(messageLength) {
				// Need more data
				break
			}

			// Extract the complete encrypted message (skip frame header)
			encryptedMessage := buffer[5 : 5+messageLength]
			buffer = buffer[5+messageLength:] // Remove processed data from buffer

			chunkCount++
			fmt.Printf("[Client Stream] Received chunk %d: compression=%d, length=%d bytes\n",
				chunkCount, compressionFlag, messageLength)

			// Send to gRPC stream
			if err := stream.SendMsg(encryptedMessage); err != nil {
				fmt.Printf("[gRPC Stream Error] Failed to send message: %v\n", err)
				http.Error(w, fmt.Sprintf("Failed to send gRPC message: %v", err), http.StatusInternalServerError)
				return
			}
		}

		// Check if we've finished reading the HTTP body
		if readErr == io.EOF {
			break
		}
		if readErr != nil {
			fmt.Printf("[HTTP Read Error] %v\n", readErr)
			http.Error(w, fmt.Sprintf("Failed to read request body: %v", readErr), http.StatusBadRequest)
			return
		}
	}

	fmt.Printf("[Client Stream] Sent %d chunks total\n", chunkCount)

	// Close the send side
	if err := stream.CloseSend(); err != nil {
		fmt.Printf("[gRPC Stream Error] Failed to close send: %v\n", err)
		http.Error(w, fmt.Sprintf("Failed to close gRPC send: %v", err), http.StatusInternalServerError)
		return
	}

	// Receive the single response
	var responseBody []byte
	if err := stream.RecvMsg(&responseBody); err != nil {
		fmt.Printf("[gRPC Stream Error] Failed to receive response: %v\n", err)
		http.Error(w, fmt.Sprintf("Failed to receive gRPC response: %v", err), http.StatusInternalServerError)
		return
	}

	fmt.Printf("[gRPC -> HTTP] Response size: %d bytes\n", len(responseBody))

	// Write the response back to the HTTP client
	w.Header().Set("Content-Type", "application/grpc")
	w.WriteHeader(http.StatusOK)
	if _, err := w.Write(responseBody); err != nil {
		fmt.Printf("[HTTP Write Error] %v\n", err)
	}
}

func handleReverseShell(w http.ResponseWriter, r *http.Request, conn *grpc.ClientConn) {
	if r.Method == http.MethodPost {
		// Handle POST: Client sending PTY output to server
		handleReverseShellPost(w, r, conn)
	} else if r.Method == http.MethodGet {
		// Handle GET: Long-polling for PTY input from server
		handleReverseShellGet(w, r, conn)
	} else {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
	}
}

func handleReverseShellPost(w http.ResponseWriter, r *http.Request, conn *grpc.ClientConn) {
	fmt.Printf("[ReverseShell POST] Sending PTY output to server\n")

	// Read the encrypted request body
	requestBody, err := io.ReadAll(r.Body)
	if err != nil {
		http.Error(w, fmt.Sprintf("Failed to read request body: %v", err), http.StatusBadRequest)
		return
	}
	defer r.Body.Close()

	// Forward to gRPC server using unary call (simpler than streaming for individual messages)
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	var responseBody []byte
	err = conn.Invoke(
		ctx,
		"/c2.C2/ReverseShell",
		requestBody,
		&responseBody,
		grpc.CallContentSubtype("raw"),
	)

	if err != nil {
		fmt.Printf("[gRPC Error] %v\n", err)
		http.Error(w, fmt.Sprintf("gRPC call failed: %v", err), http.StatusInternalServerError)
		return
	}

	// Return empty success response
	w.Header().Set("Content-Type", "application/grpc")
	w.WriteHeader(http.StatusOK)
}

func handleReverseShellGet(w http.ResponseWriter, r *http.Request, conn *grpc.ClientConn) {
	fmt.Printf("[ReverseShell GET] Long-polling for PTY input\n")

	// Create a long-running context for long polling
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	// Make a gRPC call to get pending PTY input (server should hold connection until data available)
	var responseBody []byte
	err := conn.Invoke(
		ctx,
		"/c2.C2/ReverseShell",
		[]byte{}, // Empty request for polling
		&responseBody,
		grpc.CallContentSubtype("raw"),
	)

	if err != nil {
		fmt.Printf("[gRPC Error] %v\n", err)
		http.Error(w, fmt.Sprintf("gRPC call failed: %v", err), http.StatusInternalServerError)
		return
	}

	// Return the PTY input data with gRPC framing
	w.Header().Set("Content-Type", "application/grpc")
	w.WriteHeader(http.StatusOK)

	// Write gRPC frame header
	var frameHeader [5]byte
	frameHeader[0] = 0x00 // No compression
	binary.BigEndian.PutUint32(frameHeader[1:], uint32(len(responseBody)))

	w.Write(frameHeader[:])
	w.Write(responseBody)

	fmt.Printf("[ReverseShell GET] Returned %d bytes\n", len(responseBody))
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
