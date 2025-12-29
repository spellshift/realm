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
	"time"

	"github.com/google/uuid"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"realm.pub/bin/imix_connect/portalpb"
)

// ProxyServer encapsulates the state of the SOCKS5 proxy
type ProxyServer struct {
	socksConns   sync.Map // map[string]*SocksConnection
	udpListeners sync.Map // map[string]*UDPSession

	// streamMu protects concurrent access to Send() on the stream
	streamMu sync.Mutex
	stream   portalpb.Portal_InvokePortalClient
	portalID int64
}

type SocksConnection struct {
	Conn        net.Conn
	Reassembler *StreamReassembler
	SendCh      chan *portalpb.InvokePortalRequest
}

type StreamReassembler struct {
	nextExpectedSeq uint64
	buffer          map[uint64][]byte
	maxBuffer       int
	mu              sync.Mutex
}

func NewStreamReassembler(maxBuffer int) *StreamReassembler {
	return &StreamReassembler{
		buffer:    make(map[uint64][]byte),
		maxBuffer: maxBuffer,
	}
}

// Process returns ordered data chunks. Returns error if stall detected.
func (r *StreamReassembler) Process(seqID uint64, data []byte) ([][]byte, error) {
	r.mu.Lock()
	defer r.mu.Unlock()

	var chunks [][]byte

	if seqID == r.nextExpectedSeq {
		chunks = append(chunks, data)
		r.nextExpectedSeq++

		// Check buffer for contiguous packets
		for {
			if val, ok := r.buffer[r.nextExpectedSeq]; ok {
				chunks = append(chunks, val)
				delete(r.buffer, r.nextExpectedSeq)
				r.nextExpectedSeq++
			} else {
				break
			}
		}
	} else if seqID > r.nextExpectedSeq {
		// Out of order
		if len(r.buffer) >= r.maxBuffer {
			return nil, fmt.Errorf("stall detected: buffer full (%d items) waiting for seq %d", len(r.buffer), r.nextExpectedSeq)
		}
		r.buffer[seqID] = data
	}
	// Ignore duplicates (seqID < nextExpectedSeq)

	return chunks, nil
}

type UDPSession struct {
	Conn       *net.UDPConn
	ClientAddr *net.UDPAddr
}

func main() {
	serverAddr := flag.String("server", "127.0.0.1:8000", "Tavern server address (e.g. 127.0.0.1:8000)")
	portalID := flag.Int64("portal_id", 0, "Portal ID to connect to")
	listenAddr := flag.String("listen", "127.0.0.1:1080", "Local SOCKS5 listen address")
	flag.Parse()

	if *portalID == 0 {
		log.Fatal("portal_id is required")
	}

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

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
	log.Printf("Connecting to Tavern at %s...", serverAddr)
	conn, err := grpc.NewClient(
		serverAddr,
		grpc.WithTransportCredentials(insecure.NewCredentials()),
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

	log.Printf("Registering with Portal ID %d...", s.portalID)
	err = s.stream.Send(&portalpb.InvokePortalRequest{
		PortalId: s.portalID,
	})
	if err != nil {
		return fmt.Errorf("failed to send registration message: %v", err)
	}

	errChan := make(chan error, 1)
	go func() {
		errChan <- s.handleInboundStream()
	}()

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

	go func() {
		for {
			clientConn, err := listener.Accept()
			if err != nil {
				if ctx.Err() != nil {
					return
				}
				log.Printf("Accept error: %v", err)
				continue
			}
			go s.handleSocksConnection(clientConn)
		}
	}()

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

		payload := resp.GetPayload()
		if payload == nil {
			slog.Warn("received message with nil payload")
			continue
		}

		if tcpMsg := payload.GetTcp(); tcpMsg != nil {
			connID := tcpMsg.GetSrcId()
			slog.Info("← Received TCP from Upstream", "first_byte", tcpMsg.GetData()[0], "data_len", len(tcpMsg.GetData()), "conn_id", connID, "seq_id", tcpMsg.GetSeqId())

			if val, ok := s.socksConns.Load(connID); ok {
				sc := val.(*SocksConnection)

				chunks, err := sc.Reassembler.Process(tcpMsg.GetSeqId(), tcpMsg.GetData())
				if err != nil {
					slog.Error("reassembly failed (stall)", "conn_id", connID, "error", err)
					sc.Conn.Close()
					s.socksConns.Delete(connID)
					continue
				}

				for _, chunk := range chunks {
					_, writeErr := sc.Conn.Write(chunk)
					if writeErr != nil {
						slog.Error("write to SOCKS client failed", "conn_id", connID, "error", writeErr)
						sc.Conn.Close()
						s.socksConns.Delete(connID)
						break
					}
					slog.Info("→ Sent TCP to Client", "data_len", len(chunk), "conn_id", connID)
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

				header := make([]byte, 0, 10+len(udpMsg.GetData()))
				header = append(header, 0x00, 0x00, 0x00)

				ip := net.ParseIP(dstAddr)
				if ip4 := ip.To4(); ip4 != nil {
					header = append(header, 0x01)
					header = append(header, ip4...)
				} else if ip6 := ip.To16(); ip6 != nil {
					header = append(header, 0x04)
					header = append(header, ip6...)
				} else {
					header = append(header, 0x01, 0, 0, 0, 0)
				}

				header = append(header, byte(dstPort>>8), byte(dstPort))
				packet := append(header, udpMsg.GetData()...)

				if session.ClientAddr != nil {
					_, err := session.Conn.WriteToUDP(packet, session.ClientAddr)
					if err != nil {
						slog.Error("write to UDP client failed", "conn_id", connID, "error", err)
					}
				}
			}
		} else if bytesMsg := payload.GetBytes(); bytesMsg != nil && bytesMsg.GetKind() == portalpb.BytesMessageKind_BYTES_MESSAGE_KIND_PING {
			slog.Debug("received ping from server")
		} else {
			slog.Warn("received unexpected payload type", "payload", payload)
		}
	}
}

func (s *ProxyServer) handleSocksConnection(conn net.Conn) {
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
	header := make([]byte, 2)
	if _, err := io.ReadFull(conn, header); err != nil {
		return
	}
	ver := header[0]
	if ver != 0x05 {
		return
	}
	nMethods := int(header[1])
	methods := make([]byte, nMethods)
	if _, err := io.ReadFull(conn, methods); err != nil {
		return
	}
	if _, err := conn.Write([]byte{0x05, 0x00}); err != nil {
		return
	}

	buf := make([]byte, 4)
	if _, err := io.ReadFull(conn, buf); err != nil {
		return
	}
	if buf[0] != 0x05 {
		reply(conn, 0x01)
		return
	}
	cmd := buf[1]
	atyp := buf[3]
	var dstAddr string

	slog.Info("SOCKS5 command received", "id", id, "cmd", cmd, "atyp", atyp)

	switch atyp {
	case 0x01:
		ipBuf := make([]byte, 4)
		if _, err := io.ReadFull(conn, ipBuf); err != nil {
			reply(conn, 0x01)
			return
		}
		dstAddr = net.IP(ipBuf).String()
	case 0x03:
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
		slog.Info("SOCKS5 resolved domain address", "domain", dstAddr)
	case 0x04:
		ipBuf := make([]byte, 16)
		if _, err := io.ReadFull(conn, ipBuf); err != nil {
			reply(conn, 0x01)
			return
		}
		dstAddr = net.IP(ipBuf).String()
	default:
		reply(conn, 0x08)
		return
	}

	portBuf := make([]byte, 2)
	if _, err := io.ReadFull(conn, portBuf); err != nil {
		reply(conn, 0x01)
		return
	}
	dstPort := binary.BigEndian.Uint16(portBuf)

	if cmd == 0x01 { // CONNECT
		reply(conn, 0x00)
		slog.Info("SOCKS5 CONNECT established (TCP)", "id", id, "dst_addr", dstAddr, "dst_port", dstPort)

		// Setup Worker Pool for Sending
		sendCh := make(chan *portalpb.InvokePortalRequest, 50)

		sc := &SocksConnection{
			Conn:        conn,
			Reassembler: NewStreamReassembler(100),
			SendCh:      sendCh,
		}
		s.socksConns.Store(id, sc)

		// Spawn sender worker
		go func() {
			for msg := range sendCh {
				if err := s.sendRequest(msg); err != nil {
					log.Printf("[%s] Failed to send to Tavern: %v", id, err)
					return
				}
			}
		}()

		buffer := make([]byte, 32*1024)
		var seqID uint64 = 0

		for {
			n, connReadErr := conn.Read(buffer)
			if n > 0 {
				slog.Info("← Received TCP from Client", "bytes", n, "first_byte", buffer[0])
				msg := &portalpb.InvokePortalRequest{
					Payload: &portalpb.Payload{
						Payload: &portalpb.Payload_Tcp{
							Tcp: &portalpb.TCPMessage{
								Data:    append([]byte(nil), buffer[:n]...),
								DstAddr: dstAddr,
								DstPort: uint32(dstPort),
								SrcId:   id,
								SeqId:   seqID,
							},
						},
					},
				}
				seqID++

				slog.Info("→ Sending TCP to Upstream", "conn_id", id, "seq_id", seqID-1)

				select {
				case sendCh <- msg:
				case <-time.After(0):
					sendCh <- msg
				}
			}
			if connReadErr != nil {
				if connReadErr != io.EOF {
					log.Printf("[%s] Connection read error: %v", id, connReadErr)
				}
				close(sendCh)
				return
			}
		}
	} else if cmd == 0x03 { // UDP ASSOCIATE
		udpConn, err := net.ListenUDP("udp", &net.UDPAddr{IP: net.IPv4zero, Port: 0})
		if err != nil {
			reply(conn, 0x01)
			return
		}

		session := &UDPSession{Conn: udpConn}
		s.udpListeners.Store(id, session)

		lAddr := udpConn.LocalAddr().(*net.UDPAddr)
		port := uint16(lAddr.Port)

		resp := []byte{0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0}
		resp = append(resp, byte(port>>8), byte(port))
		conn.Write(resp)

		go func() {
			buf := make([]byte, 65535)
			var seqID uint64 = 0
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

				if n < 10 {
					continue
				}

				pos := 10
				data := buf[pos:n]

				msg := &portalpb.InvokePortalRequest{
					Payload: &portalpb.Payload{
						Payload: &portalpb.Payload_Udp{
							Udp: &portalpb.UDPMessage{
								Data:    append([]byte(nil), data...),
								DstAddr: "0.0.0.0", // dummy
								DstPort: 0,
								SrcId:   id,
								SeqId:   seqID,
							},
						},
					},
				}
				seqID++
				s.sendRequest(msg)
			}
		}()

		dummy := make([]byte, 1)
		for {
			_, err := conn.Read(dummy)
			if err != nil {
				return
			}
		}

	} else {
		reply(conn, 0x07)
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
