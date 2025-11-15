package redirector

import (
	"context"
	"crypto/tls"
	"fmt"
	"io"
	"log"
	"log/slog"
	"net"
	"net/http"
	"net/url"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/encoding"
)

// Config holds configuration for the HTTP redirector
type Config struct {
	srv *http.Server
}

// SetServer sets the HTTP server configuration
func (c *Config) SetServer(srv *http.Server) {
	c.srv = srv
}

// RawCodec passes through raw bytes without marshaling/unmarshaling
type RawCodec struct{}

func (RawCodec) Marshal(v any) ([]byte, error) {
	if b, ok := v.([]byte); ok {
		return b, nil
	}
	return nil, fmt.Errorf("failed to marshal, message is %T", v)
}

func (RawCodec) Unmarshal(data []byte, v any) error {
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

// HTTPRedirectorRun starts an HTTP/1.1 to gRPC proxy/redirector
func HTTPRedirectorRun(ctx context.Context, upstream string, options ...func(*Config)) error {
	// Initialize Config
	cfg := &Config{}
	for _, opt := range options {
		opt(cfg)
	}

	// Parse host:port to determine if TLS should be used
	url, err := url.Parse(upstream)
	if err != nil {
		return fmt.Errorf("failed to parse upstream address: %v", err)
	}

	tc := credentials.NewTLS(&tls.Config{})
	port := url.Port()
	if port == "" {
		port = "443"
		if(url.Scheme == "http") {
			port = "80"
			tc = insecure.NewCredentials()
		}
	}


	conn, err := grpc.NewClient(
		url.Host,
		grpc.WithTransportCredentials(tc),
		grpc.WithContextDialer(func(ctx context.Context, _ string) (net.Conn, error) {
			// Resolve using IPv4 only (A records, not AAAA records)
			ips, err := net.DefaultResolver.LookupIP(ctx, "ip4", url.Hostname())
			if err != nil {
				return nil, err
			}
			if len(ips) == 0 {
				return nil, fmt.Errorf("no IPv4 addresses found for %s", url.Hostname())
			}

			// Force IPv4 by using "tcp4" instead of "tcp"
			dialer := &net.Dialer{}
			tcpConn, err := dialer.DialContext(ctx, "tcp4", net.JoinHostPort(ips[0].String(), port))
			if err != nil {
				return nil, err
			}

			return tcpConn, nil
		}),

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
	mux.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		handleHTTPRequest(w, r, conn)
	})

	server := &http.Server{
		Addr:    cfg.srv.Addr,
		Handler: mux,
	}


	slog.Info(fmt.Sprintf("HTTP/1.1 proxy listening on %s, forwarding to gRPC server at %s\n", server.Addr, upstream))
	if err := server.ListenAndServe(); err != nil {
		log.Fatalf("Failed to start HTTP server: %v", err)
	}

	return nil
}

func handleFetchAssetStreaming(w http.ResponseWriter, r *http.Request, conn *grpc.ClientConn) {
	if !requirePOST(w, r) {
		return
	}

	requestBody, ok := readRequestBody(w, r)
	if !ok {
		return
	}

	slog.Info(fmt.Sprintf("[HTTP -> gRPC Streaming] Method: /c2.C2/FetchAsset, Body size: %d bytes\n", len(requestBody)))

	ctx, cancel := createRequestContext(streamingTimeout)
	defer cancel()

	stream, err := createStream(ctx, conn, fetchAssetStream)
	if err != nil {
		handleStreamError(w, "Failed to create gRPC stream", err)
		return
	}

	if err := stream.SendMsg(requestBody); err != nil {
		handleStreamError(w, "Failed to send gRPC request", err)
		return
	}

	if err := stream.CloseSend(); err != nil {
		handleStreamError(w, "Failed to close gRPC send", err)
		return
	}

	setGRPCResponseHeaders(w)

	flusher, ok := getFlusher(w)
	if !ok {
		return
	}

	chunkCount := 0
	totalBytes := 0

	for {
		var responseChunk []byte
		err := stream.RecvMsg(&responseChunk)
		if err == io.EOF {
			break
		}
		if err != nil {
			slog.Debug(fmt.Sprintf("[gRPC Stream Error] Failed to receive message: %v\n", err))
			return
		}

		chunkCount++
		totalBytes += len(responseChunk)
		slog.Debug(fmt.Sprintf("[gRPC Stream] Received chunk %d: %d bytes\n", chunkCount, len(responseChunk)))

		// Write gRPC frame header
		frameHeader := NewFrameHeader(uint32(len(responseChunk)))
		encodedHeader := frameHeader.Encode()
		if _, err := w.Write(encodedHeader[:]); err != nil {
			slog.Debug(fmt.Sprintf("[HTTP Write Error] Failed to write frame header: %v\n", err))
			return
		}

		if _, err := w.Write(responseChunk); err != nil {
			slog.Debug(fmt.Sprintf("[HTTP Write Error] Failed to write chunk: %v\n", err))
			return
		}

		flusher.Flush()
	}

	slog.Debug(fmt.Sprintf("[gRPC -> HTTP] Streamed %d chunks, total %d bytes\n", chunkCount, totalBytes))
}

func handleReportFileStreaming(w http.ResponseWriter, r *http.Request, conn *grpc.ClientConn) {
	if !requirePOST(w, r) {
		return
	}

	slog.Info(("[HTTP -> gRPC Client Streaming] Method: /c2.C2/ReportFile\n"))

	ctx, cancel := createRequestContext(streamingTimeout)
	defer cancel()

	stream, err := createStream(ctx, conn, reportFileStream)
	if err != nil {
		handleStreamError(w, "Failed to create gRPC stream", err)
		return
	}

	buffer := make([]byte, 0, bufferCapacity)
	chunkCount := 0

	for {
		readBuf := make([]byte, readChunkSize)
		n, readErr := r.Body.Read(readBuf)
		if n > 0 {
			buffer = append(buffer, readBuf[:n]...)
		}

		// Process complete gRPC frames from buffer
		for {
			header, message, remaining, ok := ExtractFrame(buffer)
			if !ok {
				break
			}

			buffer = remaining
			chunkCount++
			slog.Debug(fmt.Sprintf("[Client Stream] Received chunk %d: compression=%d, length=%d bytes\n",
				chunkCount, header.CompressionFlag, header.MessageLength))

			if err := stream.SendMsg(message); err != nil {
				handleStreamError(w, "Failed to send gRPC message", err)
				return
			}
		}

		if readErr == io.EOF {
			break
		}
		if readErr != nil {
			slog.Debug(fmt.Sprintf("[HTTP Read Error] %v\n", readErr))
			http.Error(w, fmt.Sprintf("Failed to read request body: %v", readErr), http.StatusBadRequest)
			return
		}
	}

	slog.Debug(fmt.Sprintf("[Client Stream] Sent %d chunks total\n", chunkCount))

	if err := stream.CloseSend(); err != nil {
		handleStreamError(w, "Failed to close gRPC send", err)
		return
	}

	var responseBody []byte
	if err := stream.RecvMsg(&responseBody); err != nil {
		handleStreamError(w, "Failed to receive gRPC response", err)
		return
	}

	slog.Debug(fmt.Sprintf("[gRPC -> HTTP] Response size: %d bytes\n", len(responseBody)))

	setGRPCResponseHeaders(w)
	if _, err := w.Write(responseBody); err != nil {
		slog.Debug(fmt.Sprintf("[HTTP Write Error] %v\n", err))
	}
}

func handleHTTPRequest(w http.ResponseWriter, r *http.Request, conn *grpc.ClientConn) {
	if !requirePOST(w, r) {
		return
	}

	methodName := r.URL.Path
	if methodName == "" {
		http.Error(w, "Method name required in path", http.StatusBadRequest)
		return
	}

	requestBody, ok := readRequestBody(w, r)
	if !ok {
		return
	}

	slog.Info(fmt.Sprintf("[HTTP -> gRPC] Method: %s, Body size: %d bytes\n", methodName, len(requestBody)))

	ctx, cancel := createRequestContext(unaryTimeout)
	defer cancel()

	var responseBody []byte
	err := conn.Invoke(
		ctx,
		methodName,
		requestBody,
		&responseBody,
		grpc.CallContentSubtype("raw"),
	)

	if err != nil {
		slog.Error(fmt.Sprintf("[gRPC Error] %v\n", err))
		slog.Error(fmt.Sprintf("[grpc response body: %v\n]", responseBody))
		http.Error(w, fmt.Sprintf("gRPC call failed: %v", err), http.StatusInternalServerError)
		return
	}

	slog.Debug(fmt.Sprintf("[gRPC -> HTTP] Response size: %d bytes\n", len(responseBody)))

	setGRPCResponseHeaders(w)
	if _, err := w.Write(responseBody); err != nil {
		slog.Error(fmt.Sprintf("[HTTP Write Error] %v\n", err))
	}
}
