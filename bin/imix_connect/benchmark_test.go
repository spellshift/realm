package main

import (
	"context"
	"fmt"
	"io"
	"log"
	"log/slog"
	"net"
	"sync"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
	"golang.org/x/net/proxy"
	"google.golang.org/grpc"
	"realm.pub/bin/imix_connect/portalpb"
)

// MockDownloadServer implements portalpb.PortalServer and streams data
type MockDownloadServer struct {
	portalpb.UnimplementedPortalServer
	dataSize  int
	chunkSize int
	sendMu    sync.Mutex
}

func (s *MockDownloadServer) InvokePortal(stream portalpb.Portal_InvokePortalServer) error {
	// 1. Recv Registration
	_, err := stream.Recv()
	if err != nil {
		return err
	}

	// 2. Loop
	for {
		req, err := stream.Recv()
		if err == io.EOF {
			return nil
		}
		if err != nil {
			return err
		}

		// On first TCP data, start streaming the download
		if payload := req.GetPayload(); payload != nil {
			if tcp := payload.GetTcp(); tcp != nil {
				// Launch a goroutine to stream data back for this connection
				// Note: In a real server we'd track state to only reply once per "request",
				// but here we assume the client behaves well (sends one "start").
				go s.streamData(stream, tcp.SrcId, tcp.DstAddr, tcp.DstPort)
			}
		}
	}
}

func (s *MockDownloadServer) streamData(stream portalpb.Portal_InvokePortalServer, srcId string, dstAddr string, dstPort uint32) {
	dataSent := 0
	chunk := make([]byte, s.chunkSize)
	// Fill chunk with dummy data
	for i := range chunk {
		chunk[i] = byte(i % 255)
	}

	for dataSent < s.dataSize {
		n := s.chunkSize
		if s.dataSize-dataSent < n {
			n = s.dataSize - dataSent
		}

		resp := &portalpb.InvokePortalResponse{
			Payload: &portalpb.Payload{
				Payload: &portalpb.Payload_Tcp{
					Tcp: &portalpb.TCPMessage{
						Data:    chunk[:n],
						DstAddr: dstAddr,
						DstPort: dstPort,
						SrcId:   srcId,
					},
				},
			},
		}

		s.sendMu.Lock()
		err := stream.Send(resp)
		s.sendMu.Unlock()

		if err != nil {
			return
		}
		dataSent += n
	}
}

func BenchmarkImixConnectDownload(b *testing.B) {
	// Silence logs for benchmark
	log.SetOutput(io.Discard)
	slog.SetDefault(slog.New(slog.NewTextHandler(io.Discard, nil)))

	// 100MB
	totalSize := 100 * 1024 * 1024
	chunkSize := 32 * 1024 // 32KB chunks

	// 1. Start Mock Tavern Server
	lis, err := net.Listen("tcp", "127.0.0.1:0")
	require.NoError(b, err)
	s := grpc.NewServer()
	mockServer := &MockDownloadServer{
		dataSize:  totalSize,
		chunkSize: chunkSize,
	}
	portalpb.RegisterPortalServer(s, mockServer)
	go s.Serve(lis)
	defer s.Stop()

	serverAddr := lis.Addr().String()
	portalID := int64(9999)

	// 2. Start imix_connect logic
	socksPort := getFreePortB(b)
	socksAddr := fmt.Sprintf("127.0.0.1:%d", socksPort)

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// Run imix_connect in background
	go func() {
		proxy := &ProxyServer{
			portalID: portalID,
		}
		err := proxy.run(ctx, serverAddr, socksAddr)
		if err != nil && err != context.Canceled {
			// Don't log to stdout/stderr during benchmark unless it's a fatal error?
			// But we discarded output anyway.
		}
	}()

	// Wait for socks listener to be up
	require.Eventually(b, func() bool {
		conn, err := net.Dial("tcp", socksAddr)
		if err == nil {
			conn.Close()
			return true
		}
		return false
	}, 2*time.Second, 100*time.Millisecond)

	b.SetBytes(int64(totalSize))
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		// 3. Connect via SOCKS5
		dialer, err := proxy.SOCKS5("tcp", socksAddr, nil, proxy.Direct)
		require.NoError(b, err)

		targetAddr := "1.1.1.1:80"
		conn, err := dialer.Dial("tcp", targetAddr)
		require.NoError(b, err)

		// 4. Send Trigger
		_, err = conn.Write([]byte("start"))
		require.NoError(b, err)

		// 5. Consume Data
		buf := make([]byte, 32*1024)
		received := 0
		for received < totalSize {
			n, err := conn.Read(buf)
			if err != nil {
				if err == io.EOF {
					break
				}
				b.Fatal(err)
			}
			received += n
		}
		conn.Close()
		if received != totalSize {
			b.Fatalf("expected %d bytes, got %d", totalSize, received)
		}
	}
}

func getFreePortB(b *testing.B) int {
	l, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		b.Fatal(err)
	}
	defer l.Close()
	return l.Addr().(*net.TCPAddr).Port
}
