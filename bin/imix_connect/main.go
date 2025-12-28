package main

import (
	"context"
	"encoding/binary"
	"flag"
	"fmt"
	"io"
	"log"
	"log/slog"
	"net"
	"os"
	"os/signal"
	"sync"
	"sync/atomic"
	"syscall"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"realm.pub/bin/imix_connect/portalpb"
)

// ProxyServer encapsulates the state of the SOCKS5 proxy
type ProxyServer struct {
	socksConns sync.Map // map[uint32]net.Conn
	connIDSeq  uint32

	// streamMu protects concurrent access to Send() on the stream
	streamMu sync.Mutex
	stream   portalpb.Portal_InvokePortalClient
}

func main() {
	serverAddr := flag.String("server", "127.0.0.1:9090", "Tavern server address (e.g. 127.0.0.1:9090)")
	portalID := flag.Int64("portal_id", 0, "Portal ID to connect to")
	listenAddr := flag.String("listen", "127.0.0.1:1080", "Local SOCKS5 listen address")
	flag.Parse()

	if *portalID == 0 {
		log.Fatal("portal_id is required")
	}

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// Handle signals for graceful shutdown
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, syscall.SIGINT, syscall.SIGTERM)
	go func() {
		<-sigChan
		log.Println("Shutting down...")
		cancel()
	}()

	srv := &ProxyServer{}
	if err := srv.run(ctx, *serverAddr, *portalID, *listenAddr); err != nil {
		log.Fatal(err)
	}
}

func (s *ProxyServer) run(ctx context.Context, serverAddr string, portalID int64, listenAddr string) error {
	// 1. Connect to Tavern gRPC
	log.Printf("Connecting to Tavern at %s...", serverAddr)
	// We use WithBlock to ensure we fail fast if server is not up
	conn, err := grpc.DialContext(ctx, serverAddr,
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithBlock(),
	)
	if err != nil {
		return fmt.Errorf("failed to connect to Tavern: %v", err)
	}
	defer conn.Close()

	client := portalpb.NewPortalClient(conn)
	stream, err := client.InvokePortal(ctx)
	if err != nil {
		return fmt.Errorf("failed to invoke portal: %v", err)
	}
	s.stream = stream

	// 2. Send Registration Message
	log.Printf("Registering with Portal ID %d...", portalID)
	// Initial send is safe as we haven't started concurrent routines yet
	err = s.stream.Send(&portalpb.InvokePortalRequest{
		PortalId: portalID,
	})
	if err != nil {
		return fmt.Errorf("failed to send registration message: %v", err)
	}

	// 3. Start processing inbound messages from Tavern (gRPC -> SOCKS)
	errChan := make(chan error, 1)
	go func() {
		errChan <- s.handleInboundStream()
	}()

	// 4. Start SOCKS5 Listener
	listener, err := net.Listen("tcp", listenAddr)
	if err != nil {
		return fmt.Errorf("failed to listen on %s: %v", listenAddr, err)
	}
	log.Printf("SOCKS5 proxy listening on %s", listenAddr)

	go func() {
		<-ctx.Done()
		listener.Close()
		s.streamMu.Lock()
		s.stream.CloseSend()
		s.streamMu.Unlock()
	}()

	// 5. Accept loop
	go func() {
		for {
			clientConn, err := listener.Accept()
			if err != nil {
				// If closed due to ctx, ignore
				if ctx.Err() != nil {
					return
				}
				log.Printf("Accept error: %v", err)
				continue
			}
			go s.handleSocksConnection(clientConn)
		}
	}()

	// Wait for inbound stream to finish or context cancel
	select {
	case <-ctx.Done():
		return nil
	case err := <-errChan:
		return err
	}
}

func (s *ProxyServer) handleInboundStream() error {
	for {
		resp, err := s.stream.Recv()
		if err == io.EOF {
			log.Println("Tavern closed the stream (EOF)")
			return nil
		}
		if err != nil {
			return fmt.Errorf("error receiving from Tavern: %v", err)
		}

		// Dispatch based on payload type
		payload := resp.GetPayload()
		if payload == nil {
			slog.Warn("received message with nil payload")
			continue
		}

		if tcpMsg := payload.GetTcp(); tcpMsg != nil {
			// We use src_port in the *incoming* message to route back to the correct SOCKS client.
			// Per PR feedback: SrcPort is used as the session ID.
			connID := tcpMsg.GetSrcPort()

			// Try to find the connection
			if val, ok := s.socksConns.Load(connID); ok {
				conn := val.(net.Conn)
				_, writeErr := conn.Write(tcpMsg.GetData())
				if writeErr != nil {
					slog.Error("write to SOCKS client failed", "conn_id", connID, "error", writeErr)
					conn.Close()
					s.socksConns.Delete(connID)
				}
			} else {
				slog.Warn("received data for unknown connection", "conn_id", connID)
			}
		} else {
			slog.Warn("received unexpected payload type", "payload", payload)
		}
	}
}

func (s *ProxyServer) handleSocksConnection(conn net.Conn) {
	// Generate a unique ID for this connection
	id := atomic.AddUint32(&s.connIDSeq, 1)

	defer func() {
		conn.Close()
		s.socksConns.Delete(id)
	}()

	// --- SOCKS5 Handshake ---

	// 1. Version negotiation
	header := make([]byte, 2)
	if _, err := io.ReadFull(conn, header); err != nil {
		return
	}
	ver := header[0]
	if ver != 0x05 {
		return // Not SOCKS5
	}
	nMethods := int(header[1])
	methods := make([]byte, nMethods)
	if _, err := io.ReadFull(conn, methods); err != nil {
		return
	}

	// Reply: [VER, METHOD] (No Auth)
	if _, err := conn.Write([]byte{0x05, 0x00}); err != nil {
		return
	}

	// 2. Request
	// Client sends: [VER, CMD, RSV, ATYP, DST.ADDR, DST.PORT]
	buf := make([]byte, 4)
	if _, err := io.ReadFull(conn, buf); err != nil {
		return
	}

	if buf[0] != 0x05 { // VER must be 5
		reply(conn, 0x01) // General failure
		return
	}

	if buf[1] != 0x01 { // CMD must be CONNECT
		reply(conn, 0x07) // Command not supported
		return
	}

	atyp := buf[3]
	var dstAddr string

	switch atyp {
	case 0x01: // IPv4
		ipBuf := make([]byte, 4)
		if _, err := io.ReadFull(conn, ipBuf); err != nil {
			reply(conn, 0x01)
			return
		}
		dstAddr = net.IP(ipBuf).String()
	case 0x03: // Domain name
		lenBuf := make([]byte, 1)
		if _, err := io.ReadFull(conn, lenBuf); err != nil {
			reply(conn, 0x01)
			return
		}
		domainLen := int(lenBuf[0])
		domainBuf := make([]byte, domainLen)
		if _, err := io.ReadFull(conn, domainBuf); err != nil {
			reply(conn, 0x01)
			return
		}
		dstAddr = string(domainBuf)
	case 0x04: // IPv6
		ipBuf := make([]byte, 16)
		if _, err := io.ReadFull(conn, ipBuf); err != nil {
			reply(conn, 0x01)
			return
		}
		dstAddr = net.IP(ipBuf).String()
	default:
		reply(conn, 0x08) // Address type not supported
		return
	}

	portBuf := make([]byte, 2)
	if _, err := io.ReadFull(conn, portBuf); err != nil {
		reply(conn, 0x01)
		return
	}
	dstPort := binary.BigEndian.Uint16(portBuf)

	// Handshake success
	reply(conn, 0x00)

	// Register connection
	s.socksConns.Store(id, conn)

	// --- Data Forwarding Loop ---
	buffer := make([]byte, 32*1024)
	for {
		n, err := conn.Read(buffer)
		if err != nil {
			return
		}

		if n > 0 {
			msg := &portalpb.InvokePortalRequest{
				Payload: &portalpb.Payload{
					Payload: &portalpb.Payload_Tcp{
						Tcp: &portalpb.TCPMessage{
							Data:    buffer[:n],
							DstAddr: dstAddr,
							DstPort: uint32(dstPort),
							SrcPort: id, // Route back to us using this ID
						},
					},
				},
			}

			// Protect stream.Send with mutex as this is called from multiple goroutines
			s.streamMu.Lock()
			err := s.stream.Send(msg)
			s.streamMu.Unlock()

			if err != nil {
				log.Printf("[%d] Failed to send to Tavern: %v", id, err)
				return
			}
		}
	}
}

func reply(conn net.Conn, respCode byte) {
	conn.Write([]byte{0x05, respCode, 0x00, 0x01, 0, 0, 0, 0, 0, 0})
}
