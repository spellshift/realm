package http1

import (
	"fmt"
	"io"
	"log/slog"
	"net/http"

	"google.golang.org/grpc"
	"google.golang.org/grpc/metadata"
)

func handleFetchAssetStreaming(w http.ResponseWriter, r *http.Request, conn *grpc.ClientConn) {
	if !requirePOST(w, r) {
		return
	}

	requestBody, ok := readRequestBody(w, r)
	if !ok {
		return
	}

	slog.Debug(fmt.Sprintf("[HTTP1 -> gRPC Streaming] Method: /c2.C2/FetchAsset, Body size: %d bytes\n", len(requestBody)))

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
			slog.Error(fmt.Sprintf("[gRPC Stream Error] Failed to receive message: %v\n", err))
			return
		}

		chunkCount++
		totalBytes += len(responseChunk)
		slog.Debug(fmt.Sprintf("[gRPC Stream] Received chunk %d: %d bytes\n", chunkCount, len(responseChunk)))

		// Write gRPC frame header
		frameHeader := newFrameHeader(uint32(len(responseChunk)))
		encodedHeader := frameHeader.Encode()
		if _, err := w.Write(encodedHeader[:]); err != nil {
			slog.Error(fmt.Sprintf("[HTTP Write Error] Failed to write frame header: %v\n", err))
			return
		}

		if _, err := w.Write(responseChunk); err != nil {
			slog.Error(fmt.Sprintf("[HTTP Write Error] Failed to write chunk: %v\n", err))
			return
		}

		flusher.Flush()
	}

	slog.Debug(fmt.Sprintf("[gRPC -> HTTP1] Streamed %d chunks, total %d bytes\n", chunkCount, totalBytes))
}

func handleReportFileStreaming(w http.ResponseWriter, r *http.Request, conn *grpc.ClientConn) {
	if !requirePOST(w, r) {
		return
	}

	slog.Debug(("[HTTP1 -> gRPC Client Streaming] Method: /c2.C2/ReportFile\n"))

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
			header, message, remaining, ok := extractFrame(buffer)
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

	slog.Debug(fmt.Sprintf("[gRPC -> HTTP1] Response size: %d bytes\n", len(responseBody)))

	setGRPCResponseHeaders(w)
	if _, err := w.Write(responseBody); err != nil {
		slog.Debug(fmt.Sprintf("[HTTP Write Error] %v\n", err))
	}
}

func getClientIP(r *http.Request) string {
	if clientIp := r.Header.Get("x-forwarded-for"); len(clientIp) > 0 {
		return clientIp
	}
	return r.RemoteAddr
}

func handleHTTPRequest(w http.ResponseWriter, r *http.Request, conn *grpc.ClientConn) {
	if !requirePOST(w, r) {
		return
	}

	clientIp := getClientIP(r)

	methodName := r.URL.Path
	if methodName == "" {
		http.Error(w, "Method name required in path", http.StatusBadRequest)
		return
	}

	requestBody, ok := readRequestBody(w, r)
	if !ok {
		return
	}

	slog.Debug(fmt.Sprintf("[HTTP1 -> gRPC] Method: %s, Body size: %d bytes\n", methodName, len(requestBody)))

	ctx, cancel := createRequestContext(unaryTimeout)
	defer cancel()

	// Set x-redirected-for header with the client IP
	if clientIp != "" {
		md := metadata.New(map[string]string{
			"x-redirected-for": clientIp,
		})
		ctx = metadata.NewOutgoingContext(ctx, md)
	}

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

	slog.Debug(fmt.Sprintf("[gRPC -> HTTP1] Response size: %d bytes\n", len(responseBody)))

	setGRPCResponseHeaders(w)
	if _, err := w.Write(responseBody); err != nil {
		slog.Error(fmt.Sprintf("[HTTP Write Error] %v\n", err))
	}
}
