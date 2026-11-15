package http1

import (
	"bytes"
	"context"
	"io"
	"net"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/encoding"
	"google.golang.org/grpc/metadata"

	_ "realm.pub/tavern/internal/redirectors" // register "raw" gRPC codec
)

func TestGetClientIP(t *testing.T) {
	tests := []struct {
		name         string
		setupRequest func() *http.Request
		expectedIP   string
	}{
		{
			name: "X-Forwarded-For_Set",
			setupRequest: func() *http.Request {
				req := httptest.NewRequest(http.MethodPost, "/test", nil)
				req.Header.Set("X-Forwarded-For", "203.0.113.42")
				req.RemoteAddr = "192.0.2.1:12345"
				return req
			},
			expectedIP: "203.0.113.42",
		},
		{
			name: "X-Forwarded-For_Not_Set_IPv4",
			setupRequest: func() *http.Request {
				req := httptest.NewRequest(http.MethodPost, "/test", nil)
				req.RemoteAddr = "1.1.1.1:12345"
				return req
			},
			expectedIP: "1.1.1.1",
		},
		{
			name: "X-Forwarded-For_Not_Set_IPv6",
			setupRequest: func() *http.Request {
				req := httptest.NewRequest(http.MethodPost, "/test", nil)
				req.RemoteAddr = "[2001:db8::1]:12345"
				return req
			},
			expectedIP: "2001:db8::1",
		},
		{
			name: "X-Forwarded-For_Not_Set_IPv6_Localhost",
			setupRequest: func() *http.Request {
				req := httptest.NewRequest(http.MethodPost, "/test", nil)
				req.RemoteAddr = "[::1]:5000"
				return req
			},
			expectedIP: "::1",
		},
		{
			name: "X-Forwarded-For_Empty_Falls_Back_To_RemoteAddr",
			setupRequest: func() *http.Request {
				req := httptest.NewRequest(http.MethodPost, "/test", nil)
				req.Header.Set("X-Forwarded-For", "")
				req.RemoteAddr = "1.1.1.1:12345"
				return req
			},
			expectedIP: "1.1.1.1",
		},
		{
			name: "X-Forwarded-For_With_Multiple_IPs",
			setupRequest: func() *http.Request {
				req := httptest.NewRequest(http.MethodPost, "/test", nil)
				req.Header.Set("X-Forwarded-For", "203.0.113.42, 198.51.100.1, 192.0.2.5")
				req.RemoteAddr = "192.0.2.1:12345"
				return req
			},
			expectedIP: "203.0.113.42",
		},
		{
			name: "X-Forwarded-For_Malformed_IP",
			setupRequest: func() *http.Request {
				req := httptest.NewRequest(http.MethodPost, "/test", nil)
				req.Header.Set("X-Forwarded-For", "not-an-ip")
				req.RemoteAddr = "1.1.1.1"
				return req
			},
			expectedIP: "unknown",
		},
		{
			name: "RemoteAddr_Without_Port",
			setupRequest: func() *http.Request {
				req := httptest.NewRequest(http.MethodPost, "/test", nil)
				req.RemoteAddr = "1.1.1.1"
				return req
			},
			expectedIP: "unknown",
		},
		{
			name: "RemoteAddr_Multiple_Colons_IPv6_With_Port",
			setupRequest: func() *http.Request {
				req := httptest.NewRequest(http.MethodPost, "/test", nil)
				req.RemoteAddr = "[2001:db8::1]:5000"
				return req
			},
			expectedIP: "2001:db8::1",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			req := tt.setupRequest()
			result := getClientIP(req)
			if result != tt.expectedIP {
				t.Errorf("getClientIP() = %v, want %v", result, tt.expectedIP)
			}
		})
	}
}

func TestHTTP1UpstreamTracePropagation(t *testing.T) {
	capturedMD := make(chan metadata.MD, 4)

	lis, err := net.Listen("tcp", "127.0.0.1:0")
	require.NoError(t, err)
	upstreamSrv := grpc.NewServer(
		grpc.ForceServerCodec(encoding.GetCodec("raw")),
		grpc.UnknownServiceHandler(func(srv any, stream grpc.ServerStream) error {
			md, _ := metadata.FromIncomingContext(stream.Context())
			capturedMD <- md.Copy()
			for {
				var msg []byte
				err := stream.RecvMsg(&msg)
				if err == io.EOF {
					return stream.SendMsg([]byte("pong"))
				}
				if err != nil {
					return err
				}
			}
		}),
	)
	go func() {
		_ = upstreamSrv.Serve(lis)
	}()
	defer upstreamSrv.Stop()

	conn, err := grpc.DialContext(
		context.Background(),
		lis.Addr().String(),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithDefaultCallOptions(grpc.CallContentSubtype("raw")),
	)
	require.NoError(t, err)
	defer conn.Close()

	awaitMD := func() metadata.MD {
		select {
		case md := <-capturedMD:
			return md
		case <-time.After(2 * time.Second):
			t.Fatal("timed out waiting for upstream gRPC metadata")
			return nil
		}
	}

	// 1. Unary HTTP1 request with incoming W3C traceparent and X-Forwarded-For
	const edgeTraceID = "0af7651916cd43dd8448eb211c80319c"
	req := httptest.NewRequest(http.MethodPost, "/c2.C2/ClaimTasks", bytes.NewReader([]byte("payload")))
	req.RemoteAddr = "192.0.2.10:54321"
	req.Header.Set("traceparent", "00-"+edgeTraceID+"-b7ad6b7169203331-01")
	req.Header.Set("X-Request-ID", "http-req-99")
	req.Header.Set("X-Forwarded-For", "198.51.100.88")

	w := httptest.NewRecorder()
	handleHTTPRequest(w, req, conn)
	require.Equal(t, http.StatusOK, w.Code)

	md := awaitMD()
	assert.Equal(t, []string{edgeTraceID}, md.Get("x-trace-id"))
	require.NotEmpty(t, md.Get("traceparent"))
	assert.Contains(t, md.Get("traceparent")[0], edgeTraceID)
	assert.Equal(t, []string{"http-req-99"}, md.Get("x-request-id"))
	assert.Equal(t, []string{"198.51.100.88"}, md.Get("x-redirected-for"))

	// 2. Streaming FetchAsset HTTP1 request without incoming trace headers generates fresh traceparent + x-trace-id
	reqStream := httptest.NewRequest(http.MethodPost, "/c2.C2/FetchAsset", bytes.NewReader([]byte("asset-req")))
	reqStream.RemoteAddr = "192.0.2.44:12345"

	wStream := httptest.NewRecorder()
	handleFetchAssetStreaming(wStream, reqStream, conn)
	require.Equal(t, http.StatusOK, wStream.Code)

	mdStream := awaitMD()
	require.NotEmpty(t, mdStream.Get("x-trace-id"))
	assert.Len(t, mdStream.Get("x-trace-id")[0], 32)
	require.NotEmpty(t, mdStream.Get("traceparent"))
	assert.Contains(t, mdStream.Get("traceparent")[0], mdStream.Get("x-trace-id")[0])
	require.NotEmpty(t, mdStream.Get("x-request-id"))
	assert.Equal(t, []string{"192.0.2.44"}, mdStream.Get("x-redirected-for"))
}
