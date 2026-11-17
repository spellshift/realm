package http1

import (
	"fmt"
	"io"
	"net"
	"net/http"
	"strings"

	"google.golang.org/grpc"
	"realm.pub/tavern/internal/logging"
)

func handleFetchAssetStreaming(w http.ResponseWriter, r *http.Request, conn *grpc.ClientConn) {
	if !requirePOST(w, r) {
		log.WarnContext(r.Context(), "incoming request rejected, method not allowed", "method", r.Method, "path", r.URL.Path, "source", r.RemoteAddr)
		return
	}

	requestBody, ok := readRequestBody(w, r)
	if !ok {
		log.ErrorContext(r.Context(), "incoming request failed, could not read request body", "path", r.URL.Path, "source", r.RemoteAddr)
		return
	}

	clientIP := getClientIP(r)
	ctx, cancel := prepareUpstreamContext(r, streamingTimeout, clientIP)
	defer cancel()

	log.DebugContext(ctx, "forwarding HTTP1 request", "source", clientIP, "destination", "/c2.C2/FetchAsset", "body_size", len(requestBody))

	stream, err := createStream(ctx, conn, fetchAssetStream)
	if err != nil {
		log.ErrorContext(ctx, "upstream request failed, could not create gRPC stream", "method", "/c2.C2/FetchAsset", "error", err)
		handleStreamError(w, "Failed to create gRPC stream", err)
		return
	}

	if err := stream.SendMsg(requestBody); err != nil {
		log.ErrorContext(ctx, "upstream request failed, could not send gRPC request", "method", "/c2.C2/FetchAsset", "error", err)
		handleStreamError(w, "Failed to send gRPC request", err)
		return
	}

	if err := stream.CloseSend(); err != nil {
		log.ErrorContext(ctx, "upstream request failed, could not close gRPC send", "method", "/c2.C2/FetchAsset", "error", err)
		handleStreamError(w, "Failed to close gRPC send", err)
		return
	}

	flusher, ok := getFlusher(w)
	if !ok {
		return
	}

	chunkCount := 0
	totalBytes := 0
	started := false

	for {
		var responseChunk []byte
		err := stream.RecvMsg(&responseChunk)
		if err == io.EOF {
			if !started {
				setGRPCResponseHeaders(w)
			} else {
				w.Header().Set("Grpc-Status", "0")
			}
			break
		}
		if err != nil {
			log.ErrorContext(ctx, "upstream request failed, error receiving stream message", "method", "/c2.C2/FetchAsset", "error", err)
			if !started {
				handleStreamError(w, "Failed to receive gRPC response", err)
			} else {
				w.Header().Set("Grpc-Status", "13")
				w.Header().Set("Grpc-Message", err.Error())
			}
			return
		}

		if !started {
			w.Header().Add("Trailer", "Grpc-Status")
			w.Header().Add("Trailer", "Grpc-Message")
			setGRPCResponseHeaders(w)
			started = true
		}

		chunkCount++
		totalBytes += len(responseChunk)
		log.TraceContext(ctx, "received stream chunk", "chunk", chunkCount, "chunk_size", len(responseChunk))

		// Write gRPC frame header
		frameHeader := newFrameHeader(uint32(len(responseChunk)))
		encodedHeader := frameHeader.Encode()
		if _, err := w.Write(encodedHeader[:]); err != nil {
			log.ErrorContext(ctx, "incoming request failed, could not write frame header to client", "error", err)
			return
		}

		if _, err := w.Write(responseChunk); err != nil {
			log.ErrorContext(ctx, "incoming request failed, could not write chunk to client", "error", err)
			return
		}

		flusher.Flush()
	}

	log.DebugContext(ctx, "FetchAsset streaming complete", "chunks", chunkCount, "total_bytes", totalBytes)
}

func handleShortPollStreaming(w http.ResponseWriter, r *http.Request, conn *grpc.ClientConn, cfg streamConfig) {
	if !requirePOST(w, r) {
		log.WarnContext(r.Context(), "incoming request rejected, method not allowed", "method", r.Method, "path", r.URL.Path, "source", r.RemoteAddr)
		return
	}

	clientIP := getClientIP(r)
	traceID, reqID := logging.ExtractHTTPTrace(r.Header)
	ctx := logging.WithTrace(r.Context(), traceID, reqID)

	log.DebugContext(ctx, "short poll streaming request", "method", cfg.MethodPath, "source", clientIP)

	sessionID := r.Header.Get(sessionHeader)
	if sessionID == "" {
		log.WarnContext(ctx, "missing session header", "method", cfg.MethodPath, "source", clientIP)
		http.Error(w, "Missing "+sessionHeader+" header", http.StatusBadRequest)
		return
	}

	// Parse incoming gRPC stream data
	buffer, ok := readRequestBody(w, r)
	if !ok {
		log.ErrorContext(ctx, "incoming request failed, could not read request body", "path", cfg.MethodPath, "source", clientIP)
		return
	}

	// Get or create a persistent stream session
	sess, err := sessionManager.getOrCreate(ctx, sessionID, clientIP, conn, cfg)
	if err != nil {
		log.ErrorContext(ctx, "upstream request failed, could not create gRPC stream", "method", cfg.MethodPath, "error", err)
		handleStreamError(w, "Failed to create gRPC stream", err)
		return
	}

	// Send all messages extracted from the poll body
	for {
		header, message, remaining, ok := extractFrame(buffer)
		if !ok {
			break
		}

		buffer = remaining
		log.TraceContext(ctx, "received short poll stream chunk", "compression", header.CompressionFlag, "length", header.MessageLength)

		if err := sess.send(message); err != nil {
			log.ErrorContext(ctx, "upstream request failed, could not send gRPC message", "method", cfg.MethodPath, "error", err)
			sessionManager.remove(sessionID)
			handleStreamError(w, "Failed to send gRPC message", err)
			return
		}
	}

	// Drain any buffered server responses
	responses := sess.drain()

	setGRPCResponseHeaders(w)

	totalBytes := 0
	for _, responseChunk := range responses {
		totalBytes += len(responseChunk)

		fh := newFrameHeader(uint32(len(responseChunk)))
		encodedHeader := fh.Encode()
		if _, err := w.Write(encodedHeader[:]); err != nil {
			log.ErrorContext(ctx, "incoming request failed, could not write frame header to client", "error", err)
			return
		}

		if _, err := w.Write(responseChunk); err != nil {
			log.ErrorContext(ctx, "incoming request failed, could not write chunk to client", "error", err)
			return
		}
	}
	log.TraceContext(ctx, "short poll streaming complete", "session_id", sessionID, "chunks", len(responses), "total_bytes", totalBytes)
}

func handleReportFileStreaming(w http.ResponseWriter, r *http.Request, conn *grpc.ClientConn) {
	if !requirePOST(w, r) {
		log.WarnContext(r.Context(), "incoming request rejected, method not allowed", "method", r.Method, "path", r.URL.Path, "source", r.RemoteAddr)
		return
	}

	clientIP := getClientIP(r)
	ctx, cancel := prepareUpstreamContext(r, streamingTimeout, clientIP)
	defer cancel()

	log.DebugContext(ctx, "forwarding HTTP1 request", "source", clientIP, "destination", "/c2.C2/ReportFile")

	stream, err := createStream(ctx, conn, reportFileStream)
	if err != nil {
		log.ErrorContext(ctx, "upstream request failed, could not create gRPC stream", "method", "/c2.C2/ReportFile", "error", err)
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
			header, message, remaining, ok := extractFrame(buffer)
			if !ok {
				break
			}

			buffer = remaining
			chunkCount++
			log.TraceContext(ctx, "received client stream chunk", "chunk", chunkCount, "compression", header.CompressionFlag, "length", header.MessageLength)

			if err := stream.SendMsg(message); err != nil {
				log.ErrorContext(ctx, "upstream request failed, could not send gRPC message", "method", "/c2.C2/ReportFile", "chunk", chunkCount, "error", err)
				handleStreamError(w, "Failed to send gRPC message", err)
				return
			}
		}

		if readErr == io.EOF {
			break
		}
		if readErr != nil {
			log.ErrorContext(ctx, "incoming request failed, could not read request body", "method", "/c2.C2/ReportFile", "error", readErr)
			http.Error(w, fmt.Sprintf("Failed to read request body: %v", readErr), http.StatusBadRequest)
			return
		}
	}

	log.DebugContext(ctx, "ReportFile client streaming complete", "chunks_sent", chunkCount)

	if err := stream.CloseSend(); err != nil {
		log.ErrorContext(ctx, "upstream request failed, could not close gRPC send", "method", "/c2.C2/ReportFile", "error", err)
		handleStreamError(w, "Failed to close gRPC send", err)
		return
	}

	var responseBody []byte
	if err := stream.RecvMsg(&responseBody); err != nil {
		log.ErrorContext(ctx, "upstream request failed, could not receive gRPC response", "method", "/c2.C2/ReportFile", "error", err)
		handleStreamError(w, "Failed to receive gRPC response", err)
		return
	}

	log.DebugContext(ctx, "ReportFile response", "response_size", len(responseBody))

	setGRPCResponseHeaders(w)
	if _, err := w.Write(responseBody); err != nil {
		log.ErrorContext(ctx, "incoming request failed, could not write response to client", "error", err)
	}
}

func getClientIP(r *http.Request) string {
	if forwardedFor := r.Header.Get("x-forwarded-for"); len(forwardedFor) > 0 {
		// X-Forwarded-For is a comma-separated list, the first IP is the original client
		clientIp := strings.TrimSpace(strings.Split(forwardedFor, ",")[0])
		if validateIP(clientIp) {
			return clientIp
		}
		log.WarnContext(r.Context(), "invalid x-forwarded-for client IP", "ip", clientIp)
	}

	// Fallback to RemoteAddr
	host, _, err := net.SplitHostPort(r.RemoteAddr)
	if err != nil {
		log.DebugContext(r.Context(), "failed to parse remote addr", "ip", r.RemoteAddr)
	}

	host = strings.TrimSpace(host)
	if validateIP(host) {
		return host
	}
	log.DebugContext(r.Context(), "invalid remote IP", "ip", host)
	return "unknown"
}

func validateIP(ipaddr string) bool {
	return net.ParseIP(ipaddr) != nil || ipaddr == "unknown"
}

func handleHTTPRequest(w http.ResponseWriter, r *http.Request, conn *grpc.ClientConn) {
	if !requirePOST(w, r) {
		log.WarnContext(r.Context(), "incoming request rejected, method not allowed", "method", r.Method, "path", r.URL.Path, "source", r.RemoteAddr)
		return
	}

	clientIP := getClientIP(r)

	methodName := r.URL.Path
	if methodName == "" {
		log.WarnContext(r.Context(), "incoming request failed, method name required in path", "source", clientIP)
		http.Error(w, "Method name required in path", http.StatusBadRequest)
		return
	}

	requestBody, ok := readRequestBody(w, r)
	if !ok {
		log.ErrorContext(r.Context(), "incoming request failed, could not read request body", "method", methodName, "source", clientIP)
		return
	}

	ctx, cancel := prepareUpstreamContext(r, unaryTimeout, clientIP)
	defer cancel()

	log.DebugContext(ctx, "forwarding HTTP1 request", "source", clientIP, "destination", methodName, "body_size", len(requestBody))

	var responseBody []byte
	err := conn.Invoke(
		ctx,
		methodName,
		requestBody,
		&responseBody,
		grpc.CallContentSubtype("raw"),
	)

	if err != nil {
		log.ErrorContext(ctx, "upstream request failed", "method", methodName, "error", err)
		http.Error(w, fmt.Sprintf("gRPC call failed: %v", err), http.StatusInternalServerError)
		return
	}

	log.TraceContext(ctx, "forwarded HTTP1 response", "method", methodName, "response_size", len(responseBody))

	setGRPCResponseHeaders(w)
	if _, err := w.Write(responseBody); err != nil {
		log.ErrorContext(ctx, "incoming request failed, could not write response to client", "method", methodName, "error", err)
	}
}
