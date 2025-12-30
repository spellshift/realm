package main

import (
	"context"
	"encoding/binary"
	"flag"
	"fmt"
	"io"
	"log"
	"net"
	"sync"
	"sync/atomic"

	"github.com/google/uuid"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	"realm.pub/tavern/portals/portalpb"
	"realm.pub/tavern/portals/stream"
)

const maxStreamBufferedMessages = 1024

func main() {
	portalID := flag.Int64("portal_id", 0, "Portal ID")
	listenAddr := flag.String("listen_addr", "127.0.0.1:1080", "SOCKS5 Listen Address")
	upstreamAddr := flag.String("upstream_addr", "127.0.0.1:8000", "Upstream gRPC Address")
	flag.Parse()

	if *portalID == 0 {
		log.Fatal("portal_id is required")
	}

	p := &Proxy{
		portalID:     *portalID,
		listenAddr:   *listenAddr,
		upstreamAddr: *upstreamAddr,
		streams:      make(map[string]chan *portalpb.Mote),
	}

	if err := p.Run(); err != nil {
		log.Fatalf("Proxy failed: %v", err)
	}
}

type Proxy struct {
	portalID     int64
	listenAddr   string
	upstreamAddr string

	stream portalpb.Portal_OpenPortalClient
	sendMu sync.Mutex

	streamsMu sync.RWMutex
	streams   map[string]chan *portalpb.Mote

	listener net.Listener
	wg       sync.WaitGroup
}

func (p *Proxy) Run() error {
	conn, err := grpc.NewClient(p.upstreamAddr, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		return fmt.Errorf("failed to connect to upstream: %w", err)
	}
	// defer conn.Close() // Don't close immediately in Run, but Run blocks so maybe fine?
	// Ideally Proxy lives as long as main.

	client := portalpb.NewPortalClient(conn)

	ctx := context.Background()
	streamClient, err := client.OpenPortal(ctx)
	if err != nil {
		return fmt.Errorf("failed to open portal: %w", err)
	}
	p.stream = streamClient

	go p.dispatchLoop()

	l, err := net.Listen("tcp", p.listenAddr)
	if err != nil {
		return fmt.Errorf("failed to listen on %s: %w", p.listenAddr, err)
	}
	p.listener = l
	log.Printf("SOCKS5 Proxy listening on %s, upstream %s, portal %d", p.listenAddr, p.upstreamAddr, p.portalID)

	for {
		c, err := l.Accept()
		if err != nil {
			log.Printf("Accept error: %v", err)
			continue
		}
		p.wg.Add(1)
		go p.handleConnection(c)
	}
}

func (p *Proxy) dispatchLoop() {
	for {
		resp, err := p.stream.Recv()
		if err != nil {
			log.Printf("Recv error: %v", err)
			return
		}

		mote := resp.Mote
		if mote == nil {
			continue
		}

		p.streamsMu.RLock()
		ch, ok := p.streams[mote.StreamId]
		p.streamsMu.RUnlock()

		if ok {
			select {
			case ch <- mote:
			default:
				log.Printf("Stream %s buffer full, dropping mote", mote.StreamId)
			}
		}
	}
}

func (p *Proxy) sendMote(mote *portalpb.Mote) error {
	p.sendMu.Lock()
	defer p.sendMu.Unlock()

	return p.stream.Send(&portalpb.OpenPortalRequest{
		PortalId: p.portalID,
		Mote:     mote,
	})
}

func (p *Proxy) handleConnection(conn net.Conn) {
	defer p.wg.Done()
	defer conn.Close()

	// SOCKS5 Handshake
	if err := p.handshake(conn); err != nil {
		return
	}

	// Request
	cmd, _, dstAddr, dstPort, err := p.readRequest(conn)
	if err != nil {
		return
	}

	// Prepare Stream
	streamID := uuid.New().String()
	streamCh := make(chan *portalpb.Mote, maxStreamBufferedMessages)

	p.streamsMu.Lock()
	p.streams[streamID] = streamCh
	p.streamsMu.Unlock()

	defer func() {
		p.streamsMu.Lock()
		delete(p.streams, streamID)
		p.streamsMu.Unlock()
	}()

	// Context for cancellation
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	writer := stream.NewOrderedWriter(streamID, p.sendMote)
	reader := stream.NewOrderedReader(func() (*portalpb.Mote, error) {
		select {
		case m, ok := <-streamCh:
			if !ok {
				return nil, io.EOF
			}
			return m, nil
		case <-ctx.Done():
			return nil, io.EOF
		}
	})

	switch cmd {
	case 1: // CONNECT
		p.handleTCP(ctx, cancel, conn, writer, reader, dstAddr, dstPort)
	case 3: // UDP ASSOCIATE
		p.handleUDP(ctx, cancel, conn, writer, reader)
	default:
		replyCommandNotSupported(conn)
	}
}

func (p *Proxy) handshake(conn net.Conn) error {
	buf := make([]byte, 258)
	if _, err := io.ReadFull(conn, buf[:2]); err != nil {
		return err
	}
	if buf[0] != 5 {
		return fmt.Errorf("bad version")
	}
	nMethods := int(buf[1])
	if _, err := io.ReadFull(conn, buf[:nMethods]); err != nil {
		return err
	}
	_, err := replyHandshakeSuccess(conn)
	return err
}

func (p *Proxy) readRequest(conn net.Conn) (cmd byte, atyp byte, addr string, port int, err error) {
	header := make([]byte, 4)
	if _, err := io.ReadFull(conn, header); err != nil {
		return 0, 0, "", 0, err
	}
	cmd = header[1]
	atyp = header[3]

	switch atyp {
	case 1:
		ip := make([]byte, 4)
		if _, err := io.ReadFull(conn, ip); err != nil {
			return 0, 0, "", 0, err
		}
		addr = net.IP(ip).String()
	case 3:
		lenBuf := make([]byte, 1)
		if _, err := io.ReadFull(conn, lenBuf); err != nil {
			return 0, 0, "", 0, err
		}
		dlen := int(lenBuf[0])
		domain := make([]byte, dlen)
		if _, err := io.ReadFull(conn, domain); err != nil {
			return 0, 0, "", 0, err
		}
		addr = string(domain)
	case 4:
		ip := make([]byte, 16)
		if _, err := io.ReadFull(conn, ip); err != nil {
			return 0, 0, "", 0, err
		}
		addr = net.IP(ip).String()
	default:
		replyAddrTypeNotSupported(conn)
		return 0, 0, "", 0, fmt.Errorf("unsupported atyp")
	}

	portBuf := make([]byte, 2)
	if _, err := io.ReadFull(conn, portBuf); err != nil {
		return 0, 0, "", 0, err
	}
	port = int(binary.BigEndian.Uint16(portBuf))
	return cmd, atyp, addr, port, nil
}

func (p *Proxy) handleTCP(ctx context.Context, cancel context.CancelFunc, conn net.Conn, writer *stream.OrderedWriter, reader *stream.OrderedReader, dstAddr string, dstPort int) {
	replySuccess(conn)

	var wg sync.WaitGroup
	wg.Add(2)

	// Uplink (TCP -> gRPC)
	go func() {
		defer wg.Done()
		defer cancel() // Cancel context when client disconnects
		buf := make([]byte, 32*1024)
		for {
			n, err := conn.Read(buf)
			if n > 0 {
				data := make([]byte, n)
				copy(data, buf[:n])
				if err := writer.WriteTCP(data, dstAddr, uint32(dstPort)); err != nil {
					break
				}
			}
			if err != nil {
				break
			}
		}
	}()

	// Downlink (gRPC -> TCP)
	go func() {
		defer wg.Done()
		for {
			mote, err := reader.Read()
			if err != nil {
				// Context canceled or stream error
				break
			}
			if pl, ok := mote.Payload.(*portalpb.Mote_Tcp); ok {
				if len(pl.Tcp.Data) > 0 {
					conn.Write(pl.Tcp.Data)
				}
			} else if pl, ok := mote.Payload.(*portalpb.Mote_Bytes); ok {
				if len(pl.Bytes.Data) > 0 {
					conn.Write(pl.Bytes.Data)
				}
			}
		}
		// If downstream ends, we should probably close connection
		conn.Close()
	}()

	wg.Wait()
}

func (p *Proxy) handleUDP(ctx context.Context, cancel context.CancelFunc, conn net.Conn, writer *stream.OrderedWriter, reader *stream.OrderedReader) {
	udpConn, err := net.ListenUDP("udp", &net.UDPAddr{IP: net.IPv4(0, 0, 0, 0), Port: 0})
	if err != nil {
		replyGeneralFailure(conn)
		return
	}
	defer udpConn.Close()

	lAddr := udpConn.LocalAddr().(*net.UDPAddr)
	resp := []byte{0x05, 0x00, 0x00, 0x01}
	resp = append(resp, net.IP{127, 0, 0, 1}...)
	portBytes := make([]byte, 2)
	binary.BigEndian.PutUint16(portBytes, uint16(lAddr.Port))
	resp = append(resp, portBytes...)
	conn.Write(resp)

	var clientAddr atomic.Value // net.Addr

	var wg sync.WaitGroup
	wg.Add(3) // Uplink, Downlink, Keepalive

	// Keepalive: Monitor TCP connection
	go func() {
		defer wg.Done()
		defer cancel()
		defer udpConn.Close() // Force read/write loops to error

		// Wait for EOF from TCP control connection
		buf := make([]byte, 1)
		conn.Read(buf)
	}()

	// Uplink (UDP -> gRPC)
	go func() {
		defer wg.Done()
		buf := make([]byte, 65535)
		for {
			n, addr, err := udpConn.ReadFromUDP(buf)
			if err != nil {
				break
			}
			clientAddr.Store(addr)

			if n < 3 { continue }
			pos := 3
			atyp := buf[pos]
			pos++
			var tAddr string
			var tPort int

			switch atyp {
			case 1:
				if n < pos+4+2 { continue }
				tAddr = net.IP(buf[pos : pos+4]).String()
				pos += 4
			case 3:
				if n < pos+1 { continue }
				dlen := int(buf[pos])
				pos++
				if n < pos+dlen+2 { continue }
				tAddr = string(buf[pos : pos+dlen])
				pos += dlen
			case 4:
				if n < pos+16+2 { continue }
				tAddr = net.IP(buf[pos : pos+16]).String()
				pos += 16
			default:
				continue
			}
			tPort = int(binary.BigEndian.Uint16(buf[pos : pos+2]))
			pos += 2

			data := make([]byte, n-pos)
			copy(data, buf[pos:n])

			writer.WriteUDP(data, tAddr, uint32(tPort))
		}
	}()

	// Downlink (gRPC -> UDP)
	go func() {
		defer wg.Done()
		for {
			mote, err := reader.Read()
			if err != nil {
				break
			}

			if udpMote, ok := mote.Payload.(*portalpb.Mote_Udp); ok {
				cAddrVal := clientAddr.Load()
				if cAddrVal == nil {
					continue // Drop if we don't know where to send yet
				}
				cAddr := cAddrVal.(net.Addr)

				// Header Construction
				var addrBytes []byte
				var atyp byte

				ip := net.ParseIP(udpMote.Udp.DstAddr)
				if ip != nil {
					if ip4 := ip.To4(); ip4 != nil {
						atyp = 1
						addrBytes = ip4
					} else {
						atyp = 4
						addrBytes = ip
					}
				} else {
					atyp = 3
					addrBytes = append([]byte{byte(len(udpMote.Udp.DstAddr))}, []byte(udpMote.Udp.DstAddr)...)
				}

				pkt := []byte{0x00, 0x00, 0x00, atyp}
				pkt = append(pkt, addrBytes...)
				pb := make([]byte, 2)
				binary.BigEndian.PutUint16(pb, uint16(udpMote.Udp.DstPort))
				pkt = append(pkt, pb...)
				pkt = append(pkt, udpMote.Udp.Data...)

				if uAddr, ok := cAddr.(*net.UDPAddr); ok {
					udpConn.WriteToUDP(pkt, uAddr)
				}
			}
		}
	}()

	wg.Wait()
}

// Helpers for SOCKS5 Replies

func replyHandshakeSuccess(conn net.Conn) (int, error) {
	return conn.Write([]byte{0x05, 0x00})
}

func replySuccess(conn net.Conn) (int, error) {
	return conn.Write([]byte{0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0})
}

func replyCommandNotSupported(conn net.Conn) (int, error) {
	return conn.Write([]byte{0x05, 0x07, 0x00, 0x01, 0, 0, 0, 0, 0, 0})
}

func replyAddrTypeNotSupported(conn net.Conn) (int, error) {
	return conn.Write([]byte{0x05, 0x08, 0x00, 0x01, 0, 0, 0, 0, 0, 0})
}

func replyGeneralFailure(conn net.Conn) (int, error) {
	return conn.Write([]byte{0x05, 0x01, 0x00, 0x01, 0, 0, 0, 0, 0, 0})
}
