package main

import (
	"context"
	"crypto/rand"
	"encoding/binary"
	"io"
	"net"
	"sync"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc"

	"realm.pub/tavern/portals/portalpb"
	"realm.pub/tavern/portals/stream"
)

// MockPortalServer implements the Portal service.
type MockPortalServer struct {
	portalpb.UnimplementedPortalServer
	mu sync.Mutex

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
					for i := range data {
						data[i] = byte(i)
					}

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

type overrideAddrConn struct {
	net.Conn
	localAddr  net.Addr
	remoteAddr net.Addr
}

func (c *overrideAddrConn) LocalAddr() net.Addr {
	if c.localAddr != nil {
		return c.localAddr
	}
	return c.Conn.LocalAddr()
}

func (c *overrideAddrConn) RemoteAddr() net.Addr {
	if c.remoteAddr != nil {
		return c.remoteAddr
	}
	return c.Conn.RemoteAddr()
}

func TestUDPBindIP(t *testing.T) {
	localConn := &overrideAddrConn{
		localAddr: &net.TCPAddr{IP: net.ParseIP("10.1.2.3"), Port: 1080},
	}

	tests := []struct {
		name       string
		listenAddr string
		conn       net.Conn
		want       string
	}{
		{
			name:       "explicit IPv4 loopback",
			listenAddr: "127.0.0.1:1080",
			conn:       localConn,
			want:       "127.0.0.1",
		},
		{
			name:       "explicit IPv6 loopback",
			listenAddr: "[::1]:1080",
			conn:       localConn,
			want:       "::1",
		},
		{
			name:       "explicit interface IP",
			listenAddr: "192.0.2.5:1080",
			conn:       localConn,
			want:       "192.0.2.5",
		},
		{
			name:       "wildcard falls back to TCP local addr",
			listenAddr: "0.0.0.0:1080",
			conn:       localConn,
			want:       "10.1.2.3",
		},
		{
			name:       "empty host falls back to TCP local addr",
			listenAddr: ":1080",
			conn:       localConn,
			want:       "10.1.2.3",
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			p := &Proxy{listenAddr: tc.listenAddr}
			got := p.udpBindIP(tc.conn)
			require.NotNil(t, got)
			assert.True(t, net.ParseIP(tc.want).Equal(got), "expected %s, got %s", tc.want, got.String())
			assert.False(t, got.IsUnspecified())
		})
	}
}

func buildSOCKS5UDPPacket(dstIP net.IP, dstPort uint16, payload []byte) []byte {
	pkt := []byte{0x00, 0x00, 0x00, 0x01}
	pkt = append(pkt, dstIP.To4()...)
	portBytes := make([]byte, 2)
	binary.BigEndian.PutUint16(portBytes, dstPort)
	pkt = append(pkt, portBytes...)
	pkt = append(pkt, payload...)
	return pkt
}

func TestHandleUDP_BindsListenHostAndLocksPeer(t *testing.T) {
	ln, err := net.Listen("tcp", "127.0.0.1:0")
	require.NoError(t, err)
	defer ln.Close()

	serverConnCh := make(chan net.Conn, 1)
	go func() {
		c, acceptErr := ln.Accept()
		if acceptErr == nil {
			serverConnCh <- c
		}
	}()

	clientTCP, err := net.Dial("tcp", ln.Addr().String())
	require.NoError(t, err)
	defer clientTCP.Close()

	serverTCP := <-serverConnCh
	defer serverTCP.Close()

	p := &Proxy{listenAddr: "127.0.0.1:1080"}
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	sentMotes := make(chan *portalpb.Mote, 10)
	downlinkCh := make(chan *portalpb.Mote, 10)

	writer := stream.NewOrderedWriter("udp-stream", func(m *portalpb.Mote) error {
		sentMotes <- m
		return nil
	})
	reader := stream.NewOrderedReader(func() (*portalpb.Mote, error) {
		select {
		case m := <-downlinkCh:
			return m, nil
		case <-ctx.Done():
			return nil, io.EOF
		}
	})
	defer reader.Close()

	done := make(chan struct{})
	go func() {
		defer close(done)
		p.handleUDP(ctx, cancel, serverTCP, writer, reader, "udp-stream")
	}()

	// Read SOCKS5 UDP ASSOCIATE reply: VER(1) REP(1) RSV(1) ATYP(1) BND.ADDR(4) BND.PORT(2)
	reply := make([]byte, 10)
	_, err = io.ReadFull(clientTCP, reply)
	require.NoError(t, err)
	require.Equal(t, byte(0x05), reply[0])
	require.Equal(t, byte(0x00), reply[1])
	require.Equal(t, byte(0x01), reply[3])
	relayIP := net.IP(reply[4:8])
	relayPort := int(binary.BigEndian.Uint16(reply[8:10]))
	assert.True(t, net.IPv4(127, 0, 0, 1).Equal(relayIP), "expected relay bound to 127.0.0.1, got %s", relayIP)
	require.Greater(t, relayPort, 0)

	relayAddr := &net.UDPAddr{IP: relayIP, Port: relayPort}

	// Authorized client sends first packet -> locks clientAddr to authClient's ip:port
	authClient, err := net.ListenUDP("udp", &net.UDPAddr{IP: net.IPv4(127, 0, 0, 1), Port: 0})
	require.NoError(t, err)
	defer authClient.Close()

	_, err = authClient.WriteToUDP(buildSOCKS5UDPPacket(net.IPv4(10, 0, 0, 1), 53, []byte("legit")), relayAddr)
	require.NoError(t, err)

	select {
	case m := <-sentMotes:
		require.NotNil(t, m.GetUdp())
		assert.Equal(t, []byte("legit"), m.GetUdp().Data)
	case <-time.After(2 * time.Second):
		t.Fatal("timed out waiting for authorized UDP mote")
	}

	// Second UDP socket on a different port attempts to hijack the session -> must be dropped
	hijacker, err := net.ListenUDP("udp", &net.UDPAddr{IP: net.IPv4(127, 0, 0, 1), Port: 0})
	require.NoError(t, err)
	defer hijacker.Close()

	_, err = hijacker.WriteToUDP(buildSOCKS5UDPPacket(net.IPv4(10, 0, 0, 1), 53, []byte("hijack")), relayAddr)
	require.NoError(t, err)

	select {
	case m := <-sentMotes:
		t.Fatalf("unexpected mote forwarded from hijacker socket: %v", m)
	case <-time.After(150 * time.Millisecond):
	}

	// Downlink response must still be delivered to authClient, not hijacker
	downlinkCh <- &portalpb.Mote{
		StreamId: "udp-stream",
		SeqId:    0,
		Payload: &portalpb.Mote_Udp{
			Udp: &portalpb.UDPPayload{
				DstAddr: "10.0.0.1",
				DstPort: 53,
				Data:    []byte("reply"),
			},
		},
	}

	_ = authClient.SetReadDeadline(time.Now().Add(2 * time.Second))
	buf := make([]byte, 128)
	n, _, err := authClient.ReadFromUDP(buf)
	require.NoError(t, err)
	assert.Equal(t, []byte("reply"), buf[n-len("reply"):n])

	_ = clientTCP.Close()
	select {
	case <-done:
	case <-time.After(2 * time.Second):
		t.Fatal("handleUDP did not exit after TCP control close")
	}
}

func TestHandleUDP_RejectsOffHostIP(t *testing.T) {
	clientPipe, serverPipe := net.Pipe()
	defer clientPipe.Close()
	defer serverPipe.Close()

	// Control connection appears to come from 198.51.100.10, while UDP packets arrive from 127.0.0.1
	wrappedServer := &overrideAddrConn{
		Conn:       serverPipe,
		localAddr:  &net.TCPAddr{IP: net.IPv4(127, 0, 0, 1), Port: 1080},
		remoteAddr: &net.TCPAddr{IP: net.ParseIP("198.51.100.10"), Port: 54321},
	}

	p := &Proxy{listenAddr: "127.0.0.1:1080"}
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	sentMotes := make(chan *portalpb.Mote, 4)
	writer := stream.NewOrderedWriter("udp-offhost", func(m *portalpb.Mote) error {
		sentMotes <- m
		return nil
	})
	reader := stream.NewOrderedReader(func() (*portalpb.Mote, error) {
		<-ctx.Done()
		return nil, io.EOF
	})
	defer reader.Close()

	done := make(chan struct{})
	go func() {
		defer close(done)
		p.handleUDP(ctx, cancel, wrappedServer, writer, reader, "udp-offhost")
	}()

	reply := make([]byte, 10)
	_, err := io.ReadFull(clientPipe, reply)
	require.NoError(t, err)
	relayAddr := &net.UDPAddr{
		IP:   net.IP(reply[4:8]),
		Port: int(binary.BigEndian.Uint16(reply[8:10])),
	}

	attacker, err := net.ListenUDP("udp", &net.UDPAddr{IP: net.IPv4(127, 0, 0, 1), Port: 0})
	require.NoError(t, err)
	defer attacker.Close()

	_, err = attacker.WriteToUDP(buildSOCKS5UDPPacket(net.IPv4(10, 0, 0, 1), 53, []byte("spoofed")), relayAddr)
	require.NoError(t, err)

	select {
	case m := <-sentMotes:
		t.Fatalf("unexpected mote forwarded from mismatched peer IP: %v", m)
	case <-time.After(150 * time.Millisecond):
	}

	_ = clientPipe.Close()
	select {
	case <-done:
	case <-time.After(2 * time.Second):
		t.Fatal("handleUDP did not exit after control close")
	}
}

// Helper to test if server is up
func TestProxyConnectivity(t *testing.T) {
	// ... minimal test to ensure handshake works ...
}
