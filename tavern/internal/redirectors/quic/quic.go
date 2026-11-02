package quic

import (
	"context"
	"crypto/tls"
	"encoding/binary"
	"errors"
	"fmt"
	"io"
	"log/slog"
	"strings"

	"github.com/quic-go/quic-go"
	"google.golang.org/grpc"
	"google.golang.org/grpc/metadata"
	"realm.pub/tavern/internal/redirectors"
)

func init() {
	redirectors.Register("quic", &Redirector{})
}

// Redirector is a UDP-based QUIC redirector.
type Redirector struct{}

// Redirect implements the redirectors.Redirector interface.
func (r *Redirector) Redirect(ctx context.Context, listenOn string, upstream *grpc.ClientConn, tlsConfig *tls.Config) error {
	if tlsConfig == nil {
		var err error
		tlsConfig, err = redirectors.SelfSignedTLSConfig("localhost")
		if err != nil {
			return fmt.Errorf("failed to generate self-signed TLS config: %w", err)
		}
	}

	// quic-go requires NextProtos to be non-empty.
	if len(tlsConfig.NextProtos) == 0 {
		tlsConfig.NextProtos = []string{"realm-quic"}
	}

	listener, err := quic.ListenAddr(listenOn, tlsConfig, nil)
	if err != nil {
		return fmt.Errorf("failed to listen on UDP/QUIC: %w", err)
	}
	defer listener.Close()

	slog.Info("quic redirector: started", "listen_on", listenOn)

	go func() {
		<-ctx.Done()
		listener.Close()
	}()

	for {
		conn, err := listener.Accept(ctx)
		if err != nil {
			select {
			case <-ctx.Done():
				return nil
			default:
				slog.Error("quic redirector: failed to accept connection", "error", err)
				continue
			}
		}

		go r.handleConnection(ctx, conn, upstream)
	}
}

func (r *Redirector) handleConnection(ctx context.Context, conn *quic.Conn, upstream *grpc.ClientConn) {
	clientIP := conn.RemoteAddr().String()
	slog.Info("quic redirector: client connected", "source", clientIP)
	defer func() {
		slog.Info("quic redirector: client disconnected", "source", clientIP)
	}()

	for {
		stream, err := conn.AcceptStream(ctx)
		if err != nil {
			// Connection closed or context cancelled
			return
		}

		go func(stream *quic.Stream) {
			defer stream.Close()
			if err := r.handleStream(ctx, stream, clientIP, upstream); err != nil {
				slog.Error("quic redirector: stream error", "source", clientIP, "error", err)
			}
		}(stream)
	}
}

func (r *Redirector) handleStream(ctx context.Context, stream *quic.Stream, clientIP string, upstream *grpc.ClientConn) error {
	// 1. Read 2-byte method path length
	var pathLen uint16
	if err := binary.Read(stream, binary.BigEndian, &pathLen); err != nil {
		return fmt.Errorf("failed to read method path length: %w", err)
	}

	// 2. Read method path
	pathBytes := make([]byte, pathLen)
	if _, err := io.ReadFull(stream, pathBytes); err != nil {
		return fmt.Errorf("failed to read method path: %w", err)
	}
	methodPath := string(pathBytes)

	slog.Info("quic redirector: request", "source", clientIP, "destination", methodPath)

	// Determine framing rules based on the method
	clientFramed, serverFramed := getFraming(methodPath)

	// 3. Establish stream to upstream gRPC
	streamCtx := metadata.NewOutgoingContext(ctx, metadata.MD{})
	streamCtx = redirectors.SetRedirectedForHeader(streamCtx, clientIP)

	cs, err := upstream.NewStream(streamCtx, &grpc.StreamDesc{
		StreamName:    methodPath,
		ServerStreams: true,
		ClientStreams: true,
	}, methodPath, grpc.CallContentSubtype("raw"))
	if err != nil {
		return fmt.Errorf("failed to create client stream to upstream: %w", err)
	}

	errChan := make(chan error, 2)

	// Proxy Client -> Upstream
	go func() {
		if !clientFramed {
			// Read all unframed bytes until EOF (e.g. client sent request and called finish())
			payload, err := io.ReadAll(stream)
			if err != nil {
				errChan <- err
				return
			}
			if err := cs.SendMsg(payload); err != nil {
				errChan <- err
				return
			}
			if err := cs.CloseSend(); err != nil {
				errChan <- err
				return
			}
			errChan <- nil
		} else {
			// Framed mode: read [compression_flag(1)][length(4)] header, then payload
			header := make([]byte, 5)
			for {
				if _, err := io.ReadFull(stream, header); err != nil {
					if errors.Is(err, io.EOF) || errors.Is(err, io.ErrUnexpectedEOF) {
						break
					}
					errChan <- err
					return
				}
				msgLen := binary.BigEndian.Uint32(header[1:5])
				payload := make([]byte, msgLen)
				if _, err := io.ReadFull(stream, payload); err != nil {
					errChan <- err
					return
				}
				if err := cs.SendMsg(payload); err != nil {
					errChan <- err
					return
				}
			}
			if err := cs.CloseSend(); err != nil {
				errChan <- err
				return
			}
			errChan <- nil
		}
	}()

	// Proxy Upstream -> Client
	go func() {
		if !serverFramed {
			// Expecting a single response from upstream, write it unframed to stream
			var payload []byte
			if err := cs.RecvMsg(&payload); err != nil {
				if !errors.Is(err, io.EOF) {
					errChan <- err
					return
				}
			}
			if len(payload) > 0 {
				if _, err := stream.Write(payload); err != nil {
					errChan <- err
					return
				}
			}
			errChan <- nil
		} else {
			// Framed mode: receive payloads from upstream, prepend 5-byte header, write to stream
			header := make([]byte, 5)
			for {
				var payload []byte
				if err := cs.RecvMsg(&payload); err != nil {
					if errors.Is(err, io.EOF) {
						break
					}
					errChan <- err
					return
				}
				// Format header: [compression_flag(1)][length(4)]
				header[0] = 0
				binary.BigEndian.PutUint32(header[1:5], uint32(len(payload)))
				if _, err := stream.Write(header); err != nil {
					errChan <- err
					return
				}
				if _, err := stream.Write(payload); err != nil {
					errChan <- err
					return
				}
			}
			errChan <- nil
		}
	}()

	// Wait for both directions to finish
	var firstErr error
	for i := 0; i < 2; i++ {
		if err := <-errChan; err != nil && firstErr == nil {
			firstErr = err
		}
	}

	return firstErr
}

func getFraming(method string) (clientFramed, serverFramed bool) {
	switch {
	case strings.HasSuffix(method, "ClaimTasks"),
		strings.HasSuffix(method, "ReportCredential"),
		strings.HasSuffix(method, "ReportProcessList"),
		strings.HasSuffix(method, "ReportOutput"):
		return false, false
	case strings.HasSuffix(method, "FetchAsset"):
		return false, true
	case strings.HasSuffix(method, "ReportFile"):
		return true, false
	case strings.HasSuffix(method, "CreatePortal"):
		return true, true
	default:
		return true, true
	}
}
