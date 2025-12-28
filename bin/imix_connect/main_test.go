package main

import (
	"context"
	"fmt"
	"io"
	"net"
	"sync"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"golang.org/x/net/proxy"
	"google.golang.org/grpc"
	"realm.pub/bin/imix_connect/portalpb"
)

// MockPortalServer implements portalpb.PortalServer
type MockPortalServer struct {
	portalpb.UnimplementedPortalServer
	mu             sync.Mutex
	activeStreams  map[int64]portalpb.Portal_InvokePortalServer
	receivedReg    bool
	portalID       int64
}

func (s *MockPortalServer) InvokePortal(stream portalpb.Portal_InvokePortalServer) error {
	// 1. Recv Registration
	req, err := stream.Recv()
	if err != nil {
		return err
	}
	s.mu.Lock()
	s.portalID = req.PortalId
	s.receivedReg = true
	s.activeStreams[s.portalID] = stream
	s.mu.Unlock()

	// 2. Loop
	for {
		req, err := stream.Recv()
		if err == io.EOF {
			return nil
		}
		if err != nil {
			return err
		}

		// Echo logic
		if payload := req.GetPayload(); payload != nil {
			if tcp := payload.GetTcp(); tcp != nil {
				// Echo back with slight modification (reverse bytes)
				// Use src_port as dst_port to route back to the client ID

				// Mocking a remote echo server:
				// Received Data for DstAddr:DstPort.
				// We reply FROM DstAddr:DstPort TO the client.
				// The proxy expects the reply to have DstPort = ClientID (req.SrcPort).

				replyData := reverse(tcp.Data)

				// NOTE: We now use SrcPort as the session ID carrier for the return path too,
				// matching the proxy implementation update.

				resp := &portalpb.InvokePortalResponse{
					Payload: &portalpb.Payload{
						Payload: &portalpb.Payload_Tcp{
							Tcp: &portalpb.TCPMessage{
								Data:    replyData,
								DstAddr: tcp.DstAddr,
								DstPort: tcp.DstPort, // Target port (e.g. 80)
								SrcPort: tcp.SrcPort, // Session ID (echoed back in SrcPort)
							},
						},
					},
				}
				if err := stream.Send(resp); err != nil {
					return err
				}
			}
		}
	}
}

func reverse(b []byte) []byte {
	r := make([]byte, len(b))
	for i := range b {
		r[i] = b[len(b)-1-i]
	}
	return r
}

func TestImixConnect(t *testing.T) {
	// 1. Start Mock Tavern Server
	lis, err := net.Listen("tcp", "127.0.0.1:0")
	require.NoError(t, err)
	s := grpc.NewServer()
	mockServer := &MockPortalServer{
		activeStreams: make(map[int64]portalpb.Portal_InvokePortalServer),
	}
	portalpb.RegisterPortalServer(s, mockServer)
	go s.Serve(lis)
	defer s.Stop()

	serverAddr := lis.Addr().String()
	portalID := int64(12345)

	// 2. Start imix_connect logic
	socksPort := getFreePort(t)
	socksAddr := fmt.Sprintf("127.0.0.1:%d", socksPort)

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// Run imix_connect in background
	go func() {
		// New instance for each test
		proxy := &ProxyServer{}
		err := proxy.run(ctx, serverAddr, portalID, socksAddr)
		if err != nil && err != context.Canceled {
			fmt.Printf("imix_connect exited: %v\n", err)
		}
	}()

	// Wait for socks listener to be up
	require.Eventually(t, func() bool {
		conn, err := net.Dial("tcp", socksAddr)
		if err == nil {
			conn.Close()
			return true
		}
		return false
	}, 2*time.Second, 100*time.Millisecond)

	// 3. Connect via SOCKS5
	dialer, err := proxy.SOCKS5("tcp", socksAddr, nil, proxy.Direct)
	require.NoError(t, err)

	targetAddr := "1.1.1.1:80" // Dummy target
	conn, err := dialer.Dial("tcp", targetAddr)
	require.NoError(t, err)
	defer conn.Close()

	// 4. Send Data
	msg := []byte("hello world")
	_, err = conn.Write(msg)
	require.NoError(t, err)

	// 5. Receive Echo
	buf := make([]byte, len(msg))
	_, err = io.ReadFull(conn, buf)
	require.NoError(t, err)

	expected := reverse(msg)
	assert.Equal(t, expected, buf)

	// Verify registration happened
	mockServer.mu.Lock()
	assert.True(t, mockServer.receivedReg)
	assert.Equal(t, portalID, mockServer.portalID)
	mockServer.mu.Unlock()
}

func getFreePort(t *testing.T) int {
	l, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Fatal(err)
	}
	defer l.Close()
	return l.Addr().(*net.TCPAddr).Port
}
