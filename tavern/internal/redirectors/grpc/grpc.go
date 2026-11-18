package grpc

import (
	"context"
	"crypto/tls"
	"fmt"
	"io"
	"net"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/encoding"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/c2"
	"realm.pub/tavern/internal/logging"
	"realm.pub/tavern/internal/redirectors"
)

var log = logging.For("redirectors.grpc")

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
		grpc.UnknownServiceHandler(r.handler(upstream, ctx)),
		grpc.ForceServerCodec(encoding.GetCodec("raw")),
	}
	if tlsConfig != nil {
		log.DebugContext(ctx, "TLS enabled", "listen_on", listenOn, "min_version", tlsConfig.MinVersion, "num_certificates", len(tlsConfig.Certificates))
		serverOpts = append(serverOpts, grpc.Creds(credentials.NewTLS(tlsConfig)))
	}

	s := grpc.NewServer(serverOpts...)

	log.InfoContext(ctx, "started gRPC redirector", "listen_on", listenOn)

	go func() {
		<-ctx.Done()
		s.GracefulStop()
	}()

	return s.Serve(lis)
}

func (r *Redirector) handler(upstream *grpc.ClientConn, gateContext ...context.Context) grpc.StreamHandler {
	return func(srv any, ss grpc.ServerStream) error {
		if len(gateContext) > 0 && !redirectors.Allowed(gateContext[0]) {
			return status.Error(codes.Unavailable, "redirector disabled")
		}
		ctx, _, _ := logging.ExtractGRPCTrace(ss.Context())

		fullMethodName, ok := grpc.MethodFromServerStream(ss)
		if !ok {
			log.ErrorContext(ctx, "incoming request failed, could not get method from server stream")
			return status.Errorf(codes.Internal, "failed to get method from server stream")
		}

		// Get the client's remote IP address
		clientIP := c2.GetClientIP(ctx)

		log.DebugContext(ctx, "forwarding gRPC request", "source", clientIP, "destination", fullMethodName)

		md, ok := metadata.FromIncomingContext(ctx)
		if ok {
			outMD := md.Copy()
			outMD.Delete("traceparent")
			outMD.Delete(logging.MetadataTraceIDKey)
			outMD.Delete(logging.MetadataReqIDKey)
			ctx = metadata.NewOutgoingContext(ctx, outMD)
		}

		// Set x-redirected-for header first, then append outgoing W3C trace correlation headers.
		ctx = redirectors.SetRedirectedForHeader(ctx, clientIP)
		outCtx, _, _ := logging.InjectOutgoingGRPCTrace(ctx)

		cs, err := upstream.NewStream(outCtx, &grpc.StreamDesc{
			StreamName:    fullMethodName,
			ServerStreams: true,
			ClientStreams: true,
		}, fullMethodName, grpc.CallContentSubtype("raw"))
		if err != nil {
			log.ErrorContext(ctx, "upstream request failed, could not create client stream", "method", fullMethodName, "source", clientIP, "error", err)
			return fmt.Errorf("failed to create new client stream: %w", err)
		}

		errChan := make(chan error, 2)
		go r.proxy(ctx, ss, cs, errChan, "client->upstream", fullMethodName, clientIP)
		go r.proxy(ctx, cs, ss, errChan, "upstream->client", fullMethodName, clientIP)

		err = <-errChan
		if err == io.EOF {
			err = <-errChan
		}

		if err != nil && err != io.EOF {
			log.ErrorContext(ctx, "proxy error", "method", fullMethodName, "source", clientIP, "error", err)
			return err
		}

		return nil
	}
}

func (r *Redirector) proxy(ctx context.Context, from grpc.Stream, to grpc.Stream, errChan chan<- error, direction string, method string, clientIP string) {
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
			log.ErrorContext(ctx, "proxy receive failed", "direction", direction, "method", method, "source", clientIP, "error", err)
			errChan <- err
			return
		}

		if err := to.SendMsg(msg); err != nil {
			log.ErrorContext(ctx, "proxy send failed", "direction", direction, "method", method, "source", clientIP, "error", err)
			errChan <- err
			return
		}
	}
}
