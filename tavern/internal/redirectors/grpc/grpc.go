package grpc

import (
	"context"
	"fmt"
	"io"
	"net"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
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
func (r *Redirector) Redirect(ctx context.Context, listenOn string, upstream *grpc.ClientConn) error {
	lis, err := net.Listen("tcp", listenOn)
	if err != nil {
		return fmt.Errorf("failed to listen: %w", err)
	}

	s := grpc.NewServer(
		grpc.UnknownServiceHandler(r.handler(upstream)),
		grpc.ForceServerCodec(encoding.GetCodec("raw")),
	)

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
			return status.Errorf(codes.Internal, "failed to get method from server stream")
		}

		ctx := ss.Context()
		// Get the client's remote IP address
		clientIP := c2.GetClientIP(ctx)

		md, ok := metadata.FromIncomingContext(ctx)
		if ok {
			ctx = metadata.NewOutgoingContext(ctx, md.Copy())
		}

		// Set x-redirected-for header with the client IP
		if clientIP != "" {
			outMd, _ := metadata.FromOutgoingContext(ctx)
			if outMd == nil {
				outMd = metadata.New(nil)
			}
			outMd.Set("x-redirected-for", clientIP)
			ctx = metadata.NewOutgoingContext(ctx, outMd)
		}

		cs, err := upstream.NewStream(ctx, &grpc.StreamDesc{
			StreamName:    fullMethodName,
			ServerStreams: true,
			ClientStreams: true,
		}, fullMethodName, grpc.CallContentSubtype("raw"))
		if err != nil {
			return fmt.Errorf("failed to create new client stream: %w", err)
		}

		errChan := make(chan error, 2)
		go r.proxy(ss, cs, errChan)
		go r.proxy(cs, ss, errChan)

		err = <-errChan
		if err == io.EOF {
			err = <-errChan
		}

		if err != nil && err != io.EOF {
			return err
		}

		return nil
	}
}

func (r *Redirector) proxy(from grpc.Stream, to grpc.Stream, errChan chan<- error) {
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
			errChan <- err
			return
		}

		if err := to.SendMsg(msg); err != nil {
			errChan <- err
			return
		}
	}
}
