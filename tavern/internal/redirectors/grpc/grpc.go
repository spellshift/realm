package grpc

import (
	"context"
	"crypto/tls"
	"fmt"
	"io"
	"log/slog"
	"net"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/encoding"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/c2"
	"realm.pub/tavern/internal/redirectors"
)

func init() {
	redirectors.Register("grpc", &Redirector{})
}

// Redirector is a gRPC redirector.
type Redirector struct{}

// Redirect implements the redirectors.Redirector interface.
func (r *Redirector) Redirect(ctx context.Context, listenOn string, upstream *grpc.ClientConn, tlsConfig *tls.Config) error {
	lis, err := net.Listen("tcp", listenOn)
	if err != nil {
		return fmt.Errorf("failed to listen: %w", err)
	}

	serverOpts := []grpc.ServerOption{
		grpc.UnknownServiceHandler(r.handler(upstream)),
		grpc.ForceServerCodec(encoding.GetCodec("raw")),
	}
	if tlsConfig != nil {
		slog.Debug("grpc redirector: TLS enabled", "listen_on", listenOn, "min_version", tlsConfig.MinVersion, "num_certificates", len(tlsConfig.Certificates))
		serverOpts = append(serverOpts, grpc.Creds(credentials.NewTLS(tlsConfig)))
	}

	s := grpc.NewServer(serverOpts...)

	slog.Info("grpc redirector: started", "listen_on", listenOn)

	go func() {
		<-ctx.Done()
		s.GracefulStop()
	}()

	return s.Serve(lis)
}

func (r *Redirector) handler(upstream *grpc.ClientConn) grpc.StreamHandler {
	return func(srv any, ss grpc.ServerStream) error {
		fullMethodName, ok := grpc.MethodFromServerStream(ss)
		if !ok {
			slog.Error("grpc redirector: incoming request failed, could not get method from server stream")
			return status.Errorf(codes.Internal, "failed to get method from server stream")
		}

		ctx := ss.Context()
		// Get the client's remote IP address
		clientIP := c2.GetClientIP(ctx)

		slog.Info("grpc redirector: request", "source", clientIP, "destination", fullMethodName)

		md, ok := metadata.FromIncomingContext(ctx)
		if ok {
			ctx = metadata.NewOutgoingContext(ctx, md.Copy())
		}

		// Set x-redirected-for header with the client IP
		ctx = redirectors.SetRedirectedForHeader(ctx, clientIP)

		cs, err := upstream.NewStream(ctx, &grpc.StreamDesc{
			StreamName:    fullMethodName,
			ServerStreams: true,
			ClientStreams: true,
		}, fullMethodName, grpc.CallContentSubtype("raw"))
		if err != nil {
			slog.Error("grpc redirector: upstream request failed, could not create client stream", "method", fullMethodName, "source", clientIP, "error", err)
			return fmt.Errorf("failed to create new client stream: %w", err)
		}

		errChan := make(chan error, 2)
		go r.proxy(ss, cs, errChan, "client->upstream", fullMethodName, clientIP)
		go r.proxy(cs, ss, errChan, "upstream->client", fullMethodName, clientIP)

		err = <-errChan
		if err == io.EOF {
			err = <-errChan
		}

		if err != nil && err != io.EOF {
			slog.Error("grpc redirector: proxy error", "method", fullMethodName, "source", clientIP, "error", err)
			return err
		}

		return nil
	}
}

func (r *Redirector) proxy(from grpc.Stream, to grpc.Stream, errChan chan<- error, direction string, method string, clientIP string) {
	for {
		var msg []byte
		if err := from.RecvMsg(&msg); err != nil {
			if err == io.EOF {
				if cs, ok := to.(grpc.ClientStream); ok {
					cs.CloseSend()
				}
				errChan <- io.EOF
				return
			}
			slog.Error("grpc redirector: proxy receive failed", "direction", direction, "method", method, "source", clientIP, "error", err)
			errChan <- err
			return
		}

		if err := to.SendMsg(msg); err != nil {
			slog.Error("grpc redirector: proxy send failed", "direction", direction, "method", method, "source", clientIP, "error", err)
			errChan <- err
			return
		}
	}
}
