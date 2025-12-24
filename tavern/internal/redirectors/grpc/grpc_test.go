package grpc_test

import (
	"context"
	"fmt"
	"io"
	"net"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/encoding"
	"google.golang.org/grpc/interop/grpc_testing"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/proto"

	grpcRedirector "realm.pub/tavern/internal/redirectors/grpc"
)

// setupRawUpstreamServer creates a mock gRPC server that uses a raw codec to manually
// handle requests. This simulates the upstream server that the redirector will connect to.
func setupRawUpstreamServer(t *testing.T) (string, func()) {
	t.Helper()

	lis, err := net.Listen("tcp", "127.0.0.1:0")
	require.NoError(t, err)

	handler := func(srv any, stream grpc.ServerStream) error {
		fullMethodName, ok := grpc.MethodFromServerStream(stream)
		if !ok {
			return status.Errorf(codes.Internal, "failed to get method from server stream")
		}

		if fullMethodName != "/grpc.testing.TestService/FullDuplexCall" {
			return status.Errorf(codes.Unimplemented, "method not implemented: %s", fullMethodName)
		}

		// Manually handle the bidirectional stream
		for {
			var reqBytes []byte
			if err := stream.RecvMsg(&reqBytes); err != nil {
				if err == io.EOF {
					return nil
				}
				return err
			}

			var req grpc_testing.StreamingOutputCallRequest
			if err := proto.Unmarshal(reqBytes, &req); err != nil {
				return status.Errorf(codes.Internal, "failed to unmarshal request: %v", err)
			}

			resp := &grpc_testing.StreamingOutputCallResponse{Payload: &grpc_testing.Payload{Body: req.Payload.Body}}
			respBytes, err := proto.Marshal(resp)
			if err != nil {
				return status.Errorf(codes.Internal, "failed to marshal response: %v", err)
			}

			if err := stream.SendMsg(respBytes); err != nil {
				return err
			}
		}
	}

	// The upstream server must also use the raw codec to correctly interpret the proxied stream.
	s := grpc.NewServer(
		grpc.ForceServerCodec(encoding.GetCodec("raw")),
		grpc.UnknownServiceHandler(handler),
	)

	go func() {
		if err := s.Serve(lis); err != nil && err != grpc.ErrServerStopped {
			t.Logf("test server stopped: %v", err)
		}
	}()

	return lis.Addr().String(), func() {
		s.Stop()
	}
}

func TestRedirector_FullDuplexCall(t *testing.T) {
	// 1. Setup the raw upstream test server.
	upstreamAddr, upstreamCleanup := setupRawUpstreamServer(t)
	defer upstreamCleanup()

	// 2. Setup the redirector to point to the upstream server.
	redirector := &grpcRedirector.Redirector{}
	redirectorLis, err := net.Listen("tcp", "127.0.0.1:0")
	require.NoError(t, err)
	addr := redirectorLis.Addr().String()
	redirectorLis.Close()

	upstreamConn, err := grpc.Dial(upstreamAddr, grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithDefaultCallOptions(grpc.CallContentSubtype("raw")))
	require.NoError(t, err)
	defer upstreamConn.Close()

	go func() {
		redirector.Redirect(context.Background(), addr, upstreamConn)
	}()

	// 3. Connect a client to the redirector, also using the raw codec.
	conn, err := grpc.Dial(addr, grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithDefaultCallOptions(grpc.CallContentSubtype("raw")))
	require.NoError(t, err)
	defer conn.Close()

	// 4. Perform a bidirectional streaming call through the redirector.
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	stream, err := conn.NewStream(ctx, &grpc.StreamDesc{
		ServerStreams: true,
		ClientStreams: true,
	}, "/grpc.testing.TestService/FullDuplexCall")
	require.NoError(t, err)

	for i := 0; i < 3; i++ {
		body := []byte(fmt.Sprintf("ping-%d", i))
		req := &grpc_testing.StreamingOutputCallRequest{Payload: &grpc_testing.Payload{Body: body}}
		reqBytes, err := proto.Marshal(req)
		require.NoError(t, err)

		require.NoError(t, stream.SendMsg(reqBytes))

		var respBytes []byte
		require.NoError(t, stream.RecvMsg(&respBytes))

		var resp grpc_testing.StreamingOutputCallResponse
		require.NoError(t, proto.Unmarshal(respBytes, &resp))
		require.Equal(t, body, resp.Payload.Body)
	}

	require.NoError(t, stream.CloseSend())
}

func TestRedirector_ContextCancellation(t *testing.T) {
	redirector := &grpcRedirector.Redirector{}
	redirectorLis, err := net.Listen("tcp", "127.0.0.1:0")
	require.NoError(t, err)
	addr := redirectorLis.Addr().String()
	redirectorLis.Close()

	// We don't need a real upstream for this test.
	upstreamConn, err := grpc.Dial("localhost:1", grpc.WithTransportCredentials(insecure.NewCredentials()))
	require.NoError(t, err)
	defer upstreamConn.Close()

	ctx, cancel := context.WithCancel(context.Background())

	serverErr := make(chan error)
	go func() {
		serverErr <- redirector.Redirect(ctx, addr, upstreamConn)
	}()

	// Wait a moment for the server to start listening.
	time.Sleep(100 * time.Millisecond)

	// Cancel the context, which should trigger GracefulStop.
	cancel()

	// The redirector should stop gracefully.
	select {
	case err = <-serverErr:
		// Different versions of gRPC may return either nil or ErrServerStopped on graceful shutdown.
		if err != nil {
			require.ErrorIs(t, err, grpc.ErrServerStopped, "Redirect should return grpc.ErrServerStopped on graceful stop")
		}
	case <-time.After(1 * time.Second):
		t.Fatal("server did not stop in time")
	}
}

func TestRedirector_UpstreamFailure(t *testing.T) {
	// 1. Setup the redirector without a valid upstream.
	redirector := &grpcRedirector.Redirector{}
	redirectorLis, err := net.Listen("tcp", "127.0.0.1:0")
	require.NoError(t, err)
	addr := redirectorLis.Addr().String()
	redirectorLis.Close()

	// Connect to a non-existent upstream.
	upstreamConn, err := grpc.Dial("localhost:1", grpc.WithTransportCredentials(insecure.NewCredentials()))
	require.NoError(t, err)
	defer upstreamConn.Close()

	go func() {
		redirector.Redirect(context.Background(), addr, upstreamConn)
	}()

	// 2. Connect a client to the redirector.
	conn, err := grpc.Dial(addr, grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithDefaultCallOptions(grpc.CallContentSubtype("raw")))
	require.NoError(t, err)
	defer conn.Close()

	// 3. Attempt a streaming call.
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	stream, err := conn.NewStream(ctx, &grpc.StreamDesc{
		ServerStreams: true,
		ClientStreams: true,
	}, "/grpc.testing.TestService/FullDuplexCall")
	require.NoError(t, err)

	// 4. Attempting to receive a message should fail because the upstream is down.
	var respBytes []byte
	err = stream.RecvMsg(&respBytes)

	// 5. Verify that the error is a gRPC status error with the code Unavailable.
	require.Error(t, err)
	s, ok := status.FromError(err)
	require.True(t, ok, "error should be a gRPC status error")
	require.Equal(t, codes.Unavailable, s.Code(), "error code should be Unavailable")
}
