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
	"syscall"

	"github.com/google/uuid"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"realm.pub/bin/imix_connect/portalpb"
)

// ProxyServer encapsulates the state of the SOCKS5 proxy
type ProxyServer struct {
	socksConns   sync.Map // map[string]net.Conn (TCP connections)
	udpListeners sync.Map // map[string]*UDPSession (UDP listeners associated with a session)

	// streamMu protects concurrent access to Send() on the stream
	streamMu sync.Mutex
	stream   portalpb.Portal_InvokePortalClient
	portalID int64
}

type UDPSession struct {
	Conn       *net.UDPConn
	ClientAddr *net.UDPAddr
}

func main() {
	serverAddr := flag.String("server", "http://127.0.0.1:8000", "Tavern server address (e.g. http://127.0.0.1:8000)")
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

	srv := &ProxyServer{
		portalID: *portalID,
	}
	if err := srv.run(ctx, *serverAddr, *listenAddr); err != nil {
		log.Fatal(err)
	}
}

func (s *ProxyServer) run(ctx context.Context, serverAddr string, listenAddr string) error {
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
	log.Printf("Registering with Portal ID %d...", s.portalID)
	// Initial send is safe as we haven't started concurrent routines yet
	err = s.stream.Send(&portalpb.InvokePortalRequest{
		PortalId: s.portalID,
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
			connID := tcpMsg.GetSrcId()
			if val, ok := s.socksConns.Load(connID); ok {
				conn := val.(net.Conn)
				_, writeErr := conn.Write(tcpMsg.GetData())
				if writeErr != nil {
					slog.Error("write to SOCKS client failed", "conn_id", connID, "error", writeErr)
					conn.Close()
					s.socksConns.Delete(connID)
				}
			} else {
				slog.Warn("received TCP data for unknown connection", "conn_id", connID)
			}
		} else if udpMsg := payload.GetUdp(); udpMsg != nil {
			connID := udpMsg.GetSrcId()
			if val, ok := s.udpListeners.Load(connID); ok {
				session := val.(*UDPSession)

				dstAddr := udpMsg.GetDstAddr()
				dstPort := udpMsg.GetDstPort()

				// SOCKS5 UDP header construction
				header := make([]byte, 0, 10+len(udpMsg.GetData()))
				header = append(header, 0x00, 0x00, 0x00) // RSV, FRAG

				ip := net.ParseIP(dstAddr)
				if ip4 := ip.To4(); ip4 != nil {
					header = append(header, 0x01) // IPv4
					header = append(header, ip4...)
				} else if ip6 := ip.To16(); ip6 != nil {
					header = append(header, 0x04) // IPv6
					header = append(header, ip6...)
				} else {
					// Fallback to IPv4 0.0.0.0 if parsing fails
					header = append(header, 0x01, 0, 0, 0, 0)
				}

				header = append(header, byte(dstPort>>8), byte(dstPort))
				// Append data
				packet := append(header, udpMsg.GetData()...)

				if session.ClientAddr != nil {
					_, err := session.Conn.WriteToUDP(packet, session.ClientAddr)
					if err != nil {
						slog.Error("write to UDP client failed", "conn_id", connID, "error", err)
					}
				}
			} else {
				// No listener found, drop
			}
		} else {
			slog.Warn("received unexpected payload type", "payload", payload)
		}
	}
}

func (s *ProxyServer) handleSocksConnection(conn net.Conn) {
	// Generate a unique ID for this connection
	id := uuid.NewString()

	defer func() {
		conn.Close()
		s.socksConns.Delete(id)
		if val, ok := s.udpListeners.Load(id); ok {
			session := val.(*UDPSession)
			session.Conn.Close()
			s.udpListeners.Delete(id)
		}
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

	cmd := buf[1]
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

	if cmd == 0x01 { // CONNECT (TCP)
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
								SrcId:   id, // Route back to us using this ID
							},
						},
					},
				}

				if err := s.sendRequest(msg); err != nil {
					log.Printf("[%s] Failed to send to Tavern: %v", id, err)
					return
				}
			}
		}
	} else if cmd == 0x03 { // UDP ASSOCIATE
		// 1. Setup UDP Listener
		udpConn, err := net.ListenUDP("udp", &net.UDPAddr{IP: net.IPv4zero, Port: 0})
		if err != nil {
			reply(conn, 0x01)
			return
		}

		session := &UDPSession{Conn: udpConn}
		s.udpListeners.Store(id, session)

		// 2. Reply with BND.ADDR and BND.PORT
		lAddr := udpConn.LocalAddr().(*net.UDPAddr)
		port := uint16(lAddr.Port)

		// Reply success: VER 5, REP 0, RSV 0, ATYP 1, BND.ADDR 0.0.0.0, BND.PORT
		resp := []byte{0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0}
		resp = append(resp, byte(port>>8), byte(port))
		conn.Write(resp)

		// 3. Keep TCP connection open and handle UDP packets
		go func() {
			buf := make([]byte, 65535)
			for {
				n, clientAddr, err := udpConn.ReadFromUDP(buf)
				if err != nil {
					return
				}

				if session.ClientAddr == nil {
					session.ClientAddr = clientAddr
				} else if clientAddr.String() != session.ClientAddr.String() {
					continue
				}

				// Parse SOCKS header
				if n < 10 { continue }
				if buf[0] != 0 || buf[1] != 0 { continue } // RSV must be 0
				if buf[2] != 0 { continue } // FRAG not supported

				pos := 4
				atyp := buf[3]
				var targetAddr string

				switch atyp {
				case 0x01: // IPv4
					if n < pos+4 { continue }
					targetAddr = net.IP(buf[pos:pos+4]).String()
					pos += 4
				case 0x03: // Domain
					if n < pos+1 { continue }
					l := int(buf[pos])
					pos++
					if n < pos+l { continue }
					targetAddr = string(buf[pos:pos+l])
					pos += l
				case 0x04: // IPv6
					if n < pos+16 { continue }
					targetAddr = net.IP(buf[pos:pos+16]).String()
					pos += 16
				default:
					continue
				}

				if n < pos+2 { continue }
				targetPort := binary.BigEndian.Uint16(buf[pos:pos+2])
				pos += 2

				data := buf[pos:n]

				// Send to Tavern
				msg := &portalpb.InvokePortalRequest{
					Payload: &portalpb.Payload{
						Payload: &portalpb.Payload_Udp{
							Udp: &portalpb.UDPMessage{
								Data: data,
								DstAddr: targetAddr,
								DstPort: uint32(targetPort),
								SrcId: id,
							},
						},
					},
				}

				s.sendRequest(msg)
			}
		}()

		// Block on TCP read until closed
		dummy := make([]byte, 1)
		for {
			_, err := conn.Read(dummy)
			if err != nil {
				return
			}
		}

	} else {
		reply(conn, 0x07) // Command not supported
	}
}

func (s *ProxyServer) sendRequest(req *portalpb.InvokePortalRequest) error {
    s.streamMu.Lock()
    defer s.streamMu.Unlock()
    req.PortalId = s.portalID
    return s.stream.Send(req)
}

func reply(conn net.Conn, respCode byte) {
	conn.Write([]byte{0x05, respCode, 0x00, 0x01, 0, 0, 0, 0, 0, 0})
}
