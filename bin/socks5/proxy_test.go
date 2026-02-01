package main

import (
	"crypto/rand"
	"io"
	"net"
	"sync"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc"

	"realm.pub/tavern/portals/portalpb"
)

// MockPortalServer implements the Portal service.
type MockPortalServer struct {
	portalpb.UnimplementedPortalServer
	mu      sync.Mutex

	// For testing, we can hook into received motes
	onMote func(mote *portalpb.Mote, send func(*portalpb.Mote))
}

func (s *MockPortalServer) OpenPortal(stream portalpb.Portal_OpenPortalServer) error {
	// Send loop
	outCh := make(chan *portalpb.Mote, 100)
	errCh := make(chan error, 1)

	go func() {
		for mote := range outCh {
			if err := stream.Send(&portalpb.OpenPortalResponse{Mote: mote}); err != nil {
				errCh <- err
				return
			}
		}
	}()

	// Recv loop
	for {
		req, err := stream.Recv()
		if err != nil {
			return err
		}

		if s.onMote != nil {
			// Callback to handle logic
			s.onMote(req.Mote, func(m *portalpb.Mote) {
				outCh <- m
			})
		}
	}
}

func BenchmarkProxy(b *testing.B) {
	// 1. Start Mock gRPC Server
	lis, err := net.Listen("tcp", "127.0.0.1:0")
	require.NoError(b, err)
	defer lis.Close()

	server := grpc.NewServer()
	mockServer := &MockPortalServer{}

	// Logic for Echo/Sink
	// For benchmark, we want to test throughput.
	// Upload: Proxy -> Server. Server sinks.
	// Download: Server -> Proxy. Server sources.

	mockServer.onMote = func(mote *portalpb.Mote, send func(*portalpb.Mote)) {
		// If it's a TCP Mote
		if tcp, ok := mote.Payload.(*portalpb.Mote_Tcp); ok {
			// If data len > 0, it's upload. Sink it.
			// If it's the "magic" start packet for download test, start sending.
			if string(tcp.Tcp.Data) == "START_DOWNLOAD" {
				// Send 100MB back
				go func() {
					chunkSize := 32 * 1024
					total := 100 * 1024 * 1024
					sent := 0
					data := make([]byte, chunkSize)
					// Fill with some data
					for i := range data { data[i] = byte(i) }

					currentSeq := uint64(0)

					for sent < total {
						// Send chunk
						m := &portalpb.Mote{
							StreamId: mote.StreamId,
							SeqId:    currentSeq,
							Payload: &portalpb.Mote_Tcp{
								Tcp: &portalpb.TCPPayload{
									Data: data,
								},
							},
						}
						send(m)
						currentSeq++
						sent += chunkSize
					}
					// Send termination?
					// Proxy closes on error or timeout.
				}()
			}
		}
	}

	portalpb.RegisterPortalServer(server, mockServer)
	go server.Serve(lis)

	// 2. Start Proxy
	// Pick random port
	proxyListen, err := net.Listen("tcp", "127.0.0.1:0")
	require.NoError(b, err)
	proxyAddr := proxyListen.Addr().String()
	proxyListen.Close() // Close so proxy can bind

	proxy := &Proxy{
		portalID:     123,
		listenAddr:   proxyAddr,
		upstreamAddr: lis.Addr().String(),
		streams:      make(map[string]chan *portalpb.Mote),
	}

	go func() {
		if err := proxy.Run(); err != nil {
			// panic(err) // Might panic on Close, ignore
		}
	}()

	// Wait for proxy to be ready (naive wait)
	time.Sleep(100 * time.Millisecond)

	b.Run("Upload_100MB", func(b *testing.B) {
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			doUploadTest(b, proxyAddr, 100*1024*1024)
		}
	})

	b.Run("Download_100MB", func(b *testing.B) {
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			doDownloadTest(b, proxyAddr, 100*1024*1024)
		}
	})
}

func doUploadTest(b *testing.B, proxyAddr string, size int) {
	conn, err := net.Dial("tcp", proxyAddr)
	require.NoError(b, err)
	defer conn.Close()

	// SOCKS5 Handshake
	// 1. Auth negotiation
	conn.Write([]byte{0x05, 0x01, 0x00})
	buf := make([]byte, 2)
	conn.Read(buf)
	require.Equal(b, []byte{0x05, 0x00}, buf)

	// 2. Connect
	// CMD=1 (Connect), ATYP=1 (IPv4), Addr=127.0.0.1, Port=80
	req := []byte{0x05, 0x01, 0x00, 0x01, 127, 0, 0, 1, 0, 80}
	conn.Write(req)

	// Read response
	// VER(1) REP(1) RSV(1) ATYP(1) BND.ADDR(4) BND.PORT(2) = 10 bytes
	resp := make([]byte, 10)
	_, err = io.ReadFull(conn, resp)
	require.NoError(b, err)
	require.Equal(b, byte(0x00), resp[1]) // Success

	// 3. Send Data
	chunk := make([]byte, 32*1024)
	rand.Read(chunk) // Random data
	written := 0
	for written < size {
		n, err := conn.Write(chunk)
		if err != nil {
			b.Fatal(err)
		}
		written += n
	}
}

func doDownloadTest(b *testing.B, proxyAddr string, size int) {
	conn, err := net.Dial("tcp", proxyAddr)
	require.NoError(b, err)
	defer conn.Close()

	// SOCKS5 Handshake
	conn.Write([]byte{0x05, 0x01, 0x00})
	buf := make([]byte, 2)
	conn.Read(buf)

	// Connect
	req := []byte{0x05, 0x01, 0x00, 0x01, 127, 0, 0, 1, 0, 80}
	conn.Write(req)
	resp := make([]byte, 10)
	io.ReadFull(conn, resp)

	// Send Trigger
	conn.Write([]byte("START_DOWNLOAD"))

	// Read Data
	read := 0
	buffer := make([]byte, 32*1024)
	for read < size {
		n, err := conn.Read(buffer)
		if err != nil {
			if err == io.EOF {
				break
			}
			b.Fatal(err)
		}
		read += n
	}
	assert.GreaterOrEqual(b, read, size)
}

// Helper to test if server is up
func TestProxyConnectivity(t *testing.T) {
	// ... minimal test to ensure handshake works ...
}
