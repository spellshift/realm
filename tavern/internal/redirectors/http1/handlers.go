package http1

import (
	"fmt"
	"io"
	"log/slog"
	"net"
	"net/http"
	"strings"

	"google.golang.org/grpc"
	"realm.pub/tavern/internal/redirectors"
)

func handleFetchAssetStreaming(w http.ResponseWriter, r *http.Request, conn *grpc.ClientConn) {
	if !requirePOST(w, r) {
		slog.Error("http1 redirector: incoming request rejected, method not allowed", "method", r.Method, "path", r.URL.Path, "source", r.RemoteAddr)
		return
	}

	requestBody, ok := readRequestBody(w, r)
	if !ok {
		slog.Error("http1 redirector: incoming request failed, could not read request body", "path", r.URL.Path, "source", r.RemoteAddr)
		return
	}

	clientIP := getClientIP(r)
	slog.Info("http1 redirector: request", "source", clientIP, "destination", "/c2.C2/FetchAsset")
	slog.Debug("http1 redirector: FetchAsset request details", "body_size", len(requestBody), "source", clientIP)

	ctx, cancel := createRequestContext(streamingTimeout)
	defer cancel()

	stream, err := createStream(ctx, conn, fetchAssetStream)
	if err != nil {
		slog.Error("http1 redirector: upstream request failed, could not create gRPC stream", "method", "/c2.C2/FetchAsset", "error", err)
		handleStreamError(w, "Failed to create gRPC stream", err)
		return
	}

	if err := stream.SendMsg(requestBody); err != nil {
		slog.Error("http1 redirector: upstream request failed, could not send gRPC request", "method", "/c2.C2/FetchAsset", "error", err)
		handleStreamError(w, "Failed to send gRPC request", err)
		return
	}

	if err := stream.CloseSend(); err != nil {
		slog.Error("http1 redirector: upstream request failed, could not close gRPC send", "method", "/c2.C2/FetchAsset", "error", err)
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
			slog.Error("http1 redirector: upstream request failed, error receiving stream message", "method", "/c2.C2/FetchAsset", "error", err)
			return
		}

		chunkCount++
		totalBytes += len(responseChunk)
		slog.Debug("http1 redirector: received stream chunk", "chunk", chunkCount, "chunk_size", len(responseChunk))

		// Write gRPC frame header
		frameHeader := newFrameHeader(uint32(len(responseChunk)))
		encodedHeader := frameHeader.Encode()
		if _, err := w.Write(encodedHeader[:]); err != nil {
			slog.Error("http1 redirector: incoming request failed, could not write frame header to client", "error", err)
			return
		}

		if _, err := w.Write(responseChunk); err != nil {
			slog.Error("http1 redirector: incoming request failed, could not write chunk to client", "error", err)
			return
		}

		flusher.Flush()
	}

	slog.Debug("http1 redirector: FetchAsset streaming complete", "chunks", chunkCount, "total_bytes", totalBytes)
}

func handleReportFileStreaming(w http.ResponseWriter, r *http.Request, conn *grpc.ClientConn) {
	if !requirePOST(w, r) {
		slog.Error("http1 redirector: incoming request rejected, method not allowed", "method", r.Method, "path", r.URL.Path, "source", r.RemoteAddr)
		return
	}

	clientIP := getClientIP(r)
	slog.Info("http1 redirector: request", "source", clientIP, "destination", "/c2.C2/ReportFile")
	slog.Debug("http1 redirector: ReportFile client streaming request", "source", clientIP)

	ctx, cancel := createRequestContext(streamingTimeout)
	defer cancel()

	stream, err := createStream(ctx, conn, reportFileStream)
	if err != nil {
		slog.Error("http1 redirector: upstream request failed, could not create gRPC stream", "method", "/c2.C2/ReportFile", "error", err)
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
			slog.Debug("http1 redirector: received client stream chunk", "chunk", chunkCount, "compression", header.CompressionFlag, "length", header.MessageLength)

			if err := stream.SendMsg(message); err != nil {
				slog.Error("http1 redirector: upstream request failed, could not send gRPC message", "method", "/c2.C2/ReportFile", "chunk", chunkCount, "error", err)
				handleStreamError(w, "Failed to send gRPC message", err)
				return
			}
		}

		if readErr == io.EOF {
			break
		}
		if readErr != nil {
			slog.Error("http1 redirector: incoming request failed, could not read request body", "method", "/c2.C2/ReportFile", "error", readErr)
			http.Error(w, fmt.Sprintf("Failed to read request body: %v", readErr), http.StatusBadRequest)
			return
		}
	}

	slog.Debug("http1 redirector: ReportFile client streaming complete", "chunks_sent", chunkCount)

	if err := stream.CloseSend(); err != nil {
		slog.Error("http1 redirector: upstream request failed, could not close gRPC send", "method", "/c2.C2/ReportFile", "error", err)
		handleStreamError(w, "Failed to close gRPC send", err)
		return
	}

	var responseBody []byte
	if err := stream.RecvMsg(&responseBody); err != nil {
		slog.Error("http1 redirector: upstream request failed, could not receive gRPC response", "method", "/c2.C2/ReportFile", "error", err)
		handleStreamError(w, "Failed to receive gRPC response", err)
		return
	}

	slog.Debug("http1 redirector: ReportFile response", "response_size", len(responseBody))

	setGRPCResponseHeaders(w)
	if _, err := w.Write(responseBody); err != nil {
		slog.Error("http1 redirector: incoming request failed, could not write response to client", "error", err)
	}
}

func getClientIP(r *http.Request) string {
	if forwardedFor := r.Header.Get("x-forwarded-for"); len(forwardedFor) > 0 {
		// X-Forwarded-For is a comma-separated list, the first IP is the original client
		clientIp := strings.TrimSpace(strings.Split(forwardedFor, ",")[0])
		if validateIP(clientIp) {
			return clientIp
		} else {
			slog.Error("bad forwarded for ip", "ip", clientIp)
		}
	}

	// Fallback to RemoteAddr
	host, _, err := net.SplitHostPort(r.RemoteAddr)
	if err != nil {
		slog.Error("failed to parse remote addr", "ip", r.RemoteAddr)
	}

	host = strings.TrimSpace(host)
	if validateIP(host) {
		return host
	} else {
		slog.Error("bad remote ip", "ip", host)
	}
	return "unknown"
}

func validateIP(ipaddr string) bool {
	return net.ParseIP(ipaddr) != nil || ipaddr == "unknown"
}
func handleHTTPRequest(w http.ResponseWriter, r *http.Request, conn *grpc.ClientConn) {
	if !requirePOST(w, r) {
		slog.Error("http1 redirector: incoming request rejected, method not allowed", "method", r.Method, "path", r.URL.Path, "source", r.RemoteAddr)
		return
	}

	clientIP := getClientIP(r)

	methodName := r.URL.Path
	if methodName == "" {
		slog.Error("http1 redirector: incoming request failed, method name required in path", "source", clientIP)
		http.Error(w, "Method name required in path", http.StatusBadRequest)
		return
	}

	requestBody, ok := readRequestBody(w, r)
	if !ok {
		slog.Error("http1 redirector: incoming request failed, could not read request body", "method", methodName, "source", clientIP)
		return
	}

	slog.Info("http1 redirector: request", "source", clientIP, "destination", methodName)
	slog.Debug("http1 redirector: unary request details", "method", methodName, "body_size", len(requestBody), "source", clientIP)

	ctx, cancel := createRequestContext(unaryTimeout)
	defer cancel()

	// Set x-redirected-for header with the client IP
	ctx = redirectors.SetRedirectedForHeader(ctx, clientIP)

	var responseBody []byte
	err := conn.Invoke(
		ctx,
		methodName,
		requestBody,
		&responseBody,
		grpc.CallContentSubtype("raw"),
	)

	if err != nil {
		slog.Error("http1 redirector: upstream request failed", "method", methodName, "error", err)
		http.Error(w, fmt.Sprintf("gRPC call failed: %v", err), http.StatusInternalServerError)
		return
	}

	slog.Debug("http1 redirector: response", "method", methodName, "response_size", len(responseBody))

	setGRPCResponseHeaders(w)
	if _, err := w.Write(responseBody); err != nil {
		slog.Error("http1 redirector: incoming request failed, could not write response to client", "method", methodName, "error", err)
	}
}
