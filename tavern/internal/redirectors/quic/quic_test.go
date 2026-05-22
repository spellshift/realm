package quic_test

import (
	"context"
	"crypto/tls"
	"encoding/binary"
	"fmt"
	"io"
	"net"
	"testing"
	"time"

	"github.com/quic-go/quic-go"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/encoding"
	"google.golang.org/grpc/interop/grpc_testing"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/proto"

	quicRedirector "realm.pub/tavern/internal/redirectors/quic"
)

func setupRawUpstreamServer(t *testing.T) (string, func()) {
	t.Helper()

	lis, err := net.Listen("tcp", "127.0.0.1:0")
	require.NoError(t, err)

	handler := func(srv any, stream grpc.ServerStream) error {
		fullMethodName, ok := grpc.MethodFromServerStream(stream)
		if !ok {
			return status.Errorf(codes.Internal, "failed to get method from server stream")
		}

		switch fullMethodName {
		case "/c2.C2/ClaimTasks": // Unary call: receive one, send one, close
			var reqBytes []byte
			if err := stream.RecvMsg(&reqBytes); err != nil {
				return err
			}

			// Simply echo the payload body in the response
			var req grpc_testing.SimpleRequest
			if err := proto.Unmarshal(reqBytes, &req); err != nil {
				return status.Errorf(codes.Internal, "failed to unmarshal simple request: %v", err)
			}

			resp := &grpc_testing.SimpleResponse{Payload: &grpc_testing.Payload{Body: req.Payload.Body}}
			respBytes, err := proto.Marshal(resp)
			if err != nil {
				return status.Errorf(codes.Internal, "failed to marshal simple response: %v", err)
			}

			if err := stream.SendMsg(respBytes); err != nil {
				return err
			}
			return nil

		case "/c2.C2/CreatePortal": // Bidirectional streaming call: receive and echo until EOF
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
					return status.Errorf(codes.Internal, "failed to unmarshal streaming request: %v", err)
				}

				resp := &grpc_testing.StreamingOutputCallResponse{Payload: &grpc_testing.Payload{Body: req.Payload.Body}}
				respBytes, err := proto.Marshal(resp)
				if err != nil {
					return status.Errorf(codes.Internal, "failed to marshal streaming response: %v", err)
				}

				if err := stream.SendMsg(respBytes); err != nil {
					return err
				}
			}

		default:
			return status.Errorf(codes.Unimplemented, "method not implemented: %s", fullMethodName)
		}
	}

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

func TestRedirector_QUIC_UnaryAndStreaming(t *testing.T) {
	// 1. Setup raw upstream gRPC server
	upstreamAddr, upstreamCleanup := setupRawUpstreamServer(t)
	defer upstreamCleanup()

	// 2. Setup the QUIC redirector
	redirecter := &quicRedirector.Redirector{}
	
	// Create UDP listener to find a free port
	conn, err := net.ListenUDP("udp", &net.UDPAddr{IP: net.ParseIP("127.0.0.1"), Port: 0})
	require.NoError(t, err)
	listenAddr := conn.LocalAddr().String()
	conn.Close() // Free the port so QUIC can bind

	upstreamConn, err := grpc.Dial(upstreamAddr, grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithDefaultCallOptions(grpc.CallContentSubtype("raw")))
	require.NoError(t, err)
	defer upstreamConn.Close()

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	go func() {
		err := redirecter.Redirect(ctx, listenAddr, upstreamConn, nil)
		if err != nil {
			t.Logf("redirector exited: %v", err)
		}
	}()

	// Wait for redirector to start up and bind
	time.Sleep(500 * time.Millisecond)

	// Dial redirector using QUIC
	clientTLSConfig := &tls.Config{
		InsecureSkipVerify: true,
		NextProtos:         []string{"realm-quic"},
	}

	quicConn, err := quic.DialAddr(ctx, listenAddr, clientTLSConfig, nil)
	require.NoError(t, err)
	defer quicConn.CloseWithError(0, "test ended")

	// ==========================================
	// Test Case 1: Unary Call (ClaimTasks)
	// ==========================================
	t.Run("Unary ClaimTasks", func(t *testing.T) {
		stream, err := quicConn.OpenStreamSync(ctx)
		require.NoError(t, err)
		defer stream.Close()

		method := "/c2.C2/ClaimTasks"
		methodLen := uint16(len(method))

		// 1. Write 2-byte path length and path
		err = binary.Write(stream, binary.BigEndian, methodLen)
		require.NoError(t, err)
		_, err = stream.Write([]byte(method))
		require.NoError(t, err)

		// 2. Write request payload
		body := []byte("unary-payload-body")
		req := &grpc_testing.SimpleRequest{Payload: &grpc_testing.Payload{Body: body}}
		reqBytes, err := proto.Marshal(req)
		require.NoError(t, err)

		_, err = stream.Write(reqBytes)
		require.NoError(t, err)

		// Close write half of the stream to signal EOF
		err = stream.Close()
		require.NoError(t, err)

		// 3. Read response payload until EOF
		respBytes, err := io.ReadAll(stream)
		require.NoError(t, err)

		var resp grpc_testing.SimpleResponse
		err = proto.Unmarshal(respBytes, &resp)
		require.NoError(t, err)
		require.Equal(t, body, resp.Payload.Body)
	})

	// ==========================================
	// Test Case 2: Bidirectional Streaming (CreatePortal)
	// ==========================================
	t.Run("Bidirectional Streaming CreatePortal", func(t *testing.T) {
		stream, err := quicConn.OpenStreamSync(ctx)
		require.NoError(t, err)
		defer stream.Close()

		method := "/c2.C2/CreatePortal"
		methodLen := uint16(len(method))

		// 1. Write 2-byte path length and path
		err = binary.Write(stream, binary.BigEndian, methodLen)
		require.NoError(t, err)
		_, err = stream.Write([]byte(method))
		require.NoError(t, err)

		// Send 3 framed messages
		header := make([]byte, 5)
		for i := 0; i < 3; i++ {
			body := []byte(fmt.Sprintf("streaming-msg-%d", i))
			req := &grpc_testing.StreamingOutputCallRequest{Payload: &grpc_testing.Payload{Body: body}}
			reqBytes, err := proto.Marshal(req)
			require.NoError(t, err)

			// Write gRPC header
			header[0] = 0
			binary.BigEndian.PutUint32(header[1:5], uint32(len(reqBytes)))
			_, err = stream.Write(header)
			require.NoError(t, err)
			_, err = stream.Write(reqBytes)
			require.NoError(t, err)

			// Read response header
			_, err = io.ReadFull(stream, header)
			require.NoError(t, err)
			respLen := binary.BigEndian.Uint32(header[1:5])

			// Read response body
			respBytes := make([]byte, respLen)
			_, err = io.ReadFull(stream, respBytes)
			require.NoError(t, err)

			var resp grpc_testing.StreamingOutputCallResponse
			err = proto.Unmarshal(respBytes, &resp)
			require.NoError(t, err)
			require.Equal(t, body, resp.Payload.Body)
		}

		err = stream.Close()
		require.NoError(t, err)
	})
}
