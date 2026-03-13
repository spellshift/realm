package dns

import (
	"context"
	"crypto/tls"
	"encoding/base32"
	"encoding/binary"
	"fmt"
	"hash/crc32"
	"log/slog"
	"net"
	"net/url"
	"strings"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/protobuf/proto"
	"realm.pub/tavern/internal/c2/conversation"
	"realm.pub/tavern/internal/c2/convpb"
	"realm.pub/tavern/internal/redirectors"
)

const (
	defaultUDPPort = "53"

	// DNS protocol constants
	dnsHeaderSize  = 12
	txtRecordType  = 16
	aRecordType    = 1
	aaaaRecordType = 28
	dnsClassIN     = 1
	dnsTTLSeconds  = 60

	// DNS response flags
	dnsResponseFlags = 0x8180
	dnsErrorFlags    = 0x8183
	dnsPointer       = 0xC00C

	txtMaxChunkSize = 255

	// Benign DNS response configuration
	benignARecordIP = "0.0.0.0"

	// Async protocol configuration
	MaxActiveConversations = 200000
	ConversationTTL        = 5 * time.Minute
)

func init() {
	redirectors.Register("dns", &Redirector{
		manager: conversation.NewManager(MaxActiveConversations, ConversationTTL),
	})
}

// Redirector handles DNS-based C2 communication
type Redirector struct {
	manager     *conversation.Manager
	baseDomains []string
}

func (r *Redirector) Redirect(ctx context.Context, listenOn string, upstream *grpc.ClientConn, _ *tls.Config) error {
	listenAddr, domains, err := ParseListenAddr(listenOn)
	if err != nil {
		return fmt.Errorf("failed to parse listen address: %w", err)
	}

	if len(domains) == 0 {
		return fmt.Errorf("no base domains specified in listenOn parameter")
	}

	r.baseDomains = domains

	udpAddr, err := net.ResolveUDPAddr("udp", listenAddr)
	if err != nil {
		return fmt.Errorf("failed to resolve UDP address: %w", err)
	}

	conn, err := net.ListenUDP("udp", udpAddr)
	if err != nil {
		return fmt.Errorf("failed to listen on UDP: %w", err)
	}
	defer conn.Close()

	slog.Info("dns redirector: started", "listen_on", listenAddr, "base_domains", r.baseDomains)

	buf := make([]byte, 4096)
	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
			conn.SetReadDeadline(time.Now().Add(1 * time.Second))

			n, addr, err := conn.ReadFromUDP(buf)
			if err != nil {
				if netErr, ok := err.(net.Error); ok && netErr.Timeout() {
					continue
				}
				slog.Error("dns redirector: incoming request failed, could not read UDP", "error", err)
				continue
			}

			queryCopy := make([]byte, n)
			copy(queryCopy, buf[:n])

			go r.handleDNSQuery(ctx, conn, addr, queryCopy, upstream)
		}
	}
}

// ParseListenAddr extracts address and domain parameters from listenOn string
func ParseListenAddr(listenOn string) (string, []string, error) {
	parts := strings.SplitN(listenOn, "?", 2)
	addr := parts[0]

	if !strings.Contains(addr, ":") {
		addr = net.JoinHostPort(addr, defaultUDPPort)
	}

	if len(parts) == 1 {
		return addr, nil, nil
	}

	queryParams := parts[1]
	domains := []string{}

	for _, param := range strings.Split(queryParams, "&") {
		kv := strings.SplitN(param, "=", 2)
		if len(kv) != 2 {
			continue
		}

		key := kv[0]
		value := kv[1]

		if key == "domain" && value != "" {
			decoded, err := url.QueryUnescape(value)
			if err != nil {
				return "", nil, fmt.Errorf("failed to decode domain: %w", err)
			}
			domains = append(domains, decoded)
		}
	}

	return addr, domains, nil
}

func (r *Redirector) handleDNSQuery(ctx context.Context, conn *net.UDPConn, addr *net.UDPAddr, query []byte, upstream *grpc.ClientConn) {
	if len(query) < dnsHeaderSize {
		slog.Debug("query too short")
		return
	}

	transactionID := binary.BigEndian.Uint16(query[0:2])

	domain, queryType, err := r.parseDomainNameAndType(query[dnsHeaderSize:])
	if err != nil {
		slog.Debug("failed to parse domain", "error", err)
		return
	}

	domain = strings.ToLower(domain)

	slog.Debug("dns redirector: query details", "domain", domain, "query_type", queryType, "source", addr.String())

	subdomain, err := r.extractSubdomain(domain)
	if err != nil {
		slog.Debug("domain doesn't match base domains", "domain", domain)
		r.sendErrorResponse(conn, addr, transactionID)
		return
	}

	packet, err := r.decodePacket(subdomain)
	if err != nil {
		slog.Debug("ignoring non-C2 query", "domain", domain, "error", err)
		if queryType == aRecordType {
			slog.Debug("returning benign A record for non-C2 subdomain", "domain", domain)
			r.sendDNSResponse(conn, addr, transactionID, domain, queryType, net.ParseIP(benignARecordIP).To4())
			return
		}
		r.sendErrorResponse(conn, addr, transactionID)
		return
	}

	if packet.Type == convpb.PacketType_PACKET_TYPE_UNSPECIFIED {
		slog.Debug("ignoring packet with unspecified type", "domain", domain)
		if queryType == aRecordType {
			r.sendDNSResponse(conn, addr, transactionID, domain, queryType, net.ParseIP(benignARecordIP).To4())
			return
		}
		r.sendErrorResponse(conn, addr, transactionID)
		return
	}

	if packet.Type < convpb.PacketType_PACKET_TYPE_INIT || packet.Type > convpb.PacketType_PACKET_TYPE_COMPLETE {
		slog.Debug("ignoring packet with invalid type", "type", packet.Type, "domain", domain)
		if queryType == aRecordType {
			r.sendDNSResponse(conn, addr, transactionID, domain, queryType, net.ParseIP(benignARecordIP).To4())
			return
		}
		r.sendErrorResponse(conn, addr, transactionID)
		return
	}

	slog.Debug("parsed packet", "type", packet.Type, "seq", packet.Sequence, "conv_id", packet.ConversationId)

	maxChunkSize := queryTypeToMaxChunkSize(queryType)

	var responseData []byte
	switch packet.Type {
	case convpb.PacketType_PACKET_TYPE_INIT:
		responseData, err = r.manager.HandleInit(packet)
	case convpb.PacketType_PACKET_TYPE_DATA:
		responseData, err = r.manager.HandleData(ctx, upstream, packet, maxChunkSize, redirectors.ExternalIPNoop)
	case convpb.PacketType_PACKET_TYPE_FETCH:
		responseData, err = r.manager.HandleFetch(packet)
	case convpb.PacketType_PACKET_TYPE_COMPLETE:
		responseData, err = r.manager.HandleComplete(packet)
	default:
		err = fmt.Errorf("unknown packet type: %d", packet.Type)
	}

	if err != nil {
		if strings.Contains(err.Error(), "conversation not found") {
			slog.Debug("packet for unknown conversation",
				"type", packet.Type, "conv_id", packet.ConversationId)
		} else {
			slog.Error("dns redirector: upstream request failed", "type", packet.Type, "conv_id", packet.ConversationId, "source", addr.String(), "error", err)
		}
		r.sendErrorResponse(conn, addr, transactionID)
		return
	}

	r.sendDNSResponse(conn, addr, transactionID, domain, queryType, responseData)
}

// queryTypeToMaxChunkSize maps DNS query type to the max response chunk size.
func queryTypeToMaxChunkSize(queryType uint16) int {
	switch queryType {
	case txtRecordType:
		return 400
	case aRecordType:
		return 64
	case aaaaRecordType:
		return 128
	default:
		return 400
	}
}

func (r *Redirector) extractSubdomain(domain string) (string, error) {
	domainParts := strings.Split(domain, ".")

	for _, baseDomain := range r.baseDomains {
		baseDomainParts := strings.Split(baseDomain, ".")

		if len(domainParts) <= len(baseDomainParts) {
			continue
		}

		domainSuffix := domainParts[len(domainParts)-len(baseDomainParts):]
		matched := true
		for i, part := range baseDomainParts {
			if !strings.EqualFold(part, domainSuffix[i]) {
				matched = false
				break
			}
		}

		if matched {
			subdomainParts := domainParts[:len(domainParts)-len(baseDomainParts)]
			return strings.Join(subdomainParts, "."), nil
		}
	}

	return "", fmt.Errorf("no matching base domain")
}

func (r *Redirector) decodePacket(subdomain string) (*convpb.ConvPacket, error) {
	encodedData := strings.ReplaceAll(subdomain, ".", "")

	packetData, err := base32.StdEncoding.WithPadding(base32.NoPadding).DecodeString(strings.ToUpper(encodedData))
	if err != nil {
		return nil, fmt.Errorf("failed to decode Base32 data: %w", err)
	}

	var packet convpb.ConvPacket
	if err := proto.Unmarshal(packetData, &packet); err != nil {
		return nil, fmt.Errorf("failed to unmarshal protobuf: %w", err)
	}

	if packet.Type == convpb.PacketType_PACKET_TYPE_DATA && len(packet.Data) > 0 {
		actualCRC := crc32.ChecksumIEEE(packet.Data)
		if actualCRC != packet.Crc32 {
			return nil, fmt.Errorf("CRC mismatch: expected %d, got %d", packet.Crc32, actualCRC)
		}
	}

	return &packet, nil
}

func (r *Redirector) parseDomainNameAndType(data []byte) (string, uint16, error) {
	var labels []string
	offset := 0

	for offset < len(data) {
		length := int(data[offset])
		if length == 0 {
			break
		}
		offset++

		if offset+length > len(data) {
			return "", 0, fmt.Errorf("invalid domain name")
		}

		label := string(data[offset : offset+length])
		labels = append(labels, label)
		offset += length
	}

	offset++

	if offset+2 > len(data) {
		return "", 0, fmt.Errorf("query too short for type field")
	}

	queryType := binary.BigEndian.Uint16(data[offset : offset+2])
	domain := strings.Join(labels, ".")

	return domain, queryType, nil
}

func (r *Redirector) sendDNSResponse(conn *net.UDPConn, addr *net.UDPAddr, transactionID uint16, domain string, queryType uint16, data []byte) {
	if queryType == aRecordType || queryType == aaaaRecordType {
		encoded := base32.StdEncoding.WithPadding(base32.NoPadding).EncodeToString(data)
		data = []byte(encoded)
	}

	var recordSize int
	var answerCount uint16

	switch queryType {
	case txtRecordType:
		recordSize = 0
		answerCount = 1
	case aRecordType:
		recordSize = 4
		answerCount = uint16((len(data) + recordSize - 1) / recordSize)
		if answerCount == 0 {
			answerCount = 1
		}
	case aaaaRecordType:
		recordSize = 16
		answerCount = uint16((len(data) + recordSize - 1) / recordSize)
		if answerCount == 0 {
			answerCount = 1
		}
	default:
		recordSize = 0
		answerCount = 1
	}

	response := make([]byte, 0, 512)

	response = append(response, byte(transactionID>>8), byte(transactionID))
	response = append(response, byte(dnsResponseFlags>>8), byte(dnsResponseFlags&0xFF))
	response = append(response, 0x00, 0x01)
	response = append(response, byte(answerCount>>8), byte(answerCount&0xFF))
	response = append(response, 0x00, 0x00)
	response = append(response, 0x00, 0x00)

	for _, label := range strings.Split(domain, ".") {
		if len(label) == 0 {
			continue
		}
		response = append(response, byte(len(label)))
		response = append(response, []byte(label)...)
	}
	response = append(response, 0x00)
	response = append(response, byte(queryType>>8), byte(queryType&0xFF))
	response = append(response, 0x00, byte(dnsClassIN))

	switch queryType {
	case txtRecordType:
		response = append(response, byte(dnsPointer>>8), byte(dnsPointer&0xFF))
		response = append(response, byte(queryType>>8), byte(queryType&0xFF))
		response = append(response, 0x00, byte(dnsClassIN))
		response = append(response, 0x00, 0x00, 0x00, byte(dnsTTLSeconds))

		var rdata []byte
		if len(data) == 0 {
			rdata = []byte{0x00}
		} else {
			tempData := data
			for len(tempData) > 0 {
				chunkSize := len(tempData)
				if chunkSize > txtMaxChunkSize {
					chunkSize = txtMaxChunkSize
				}
				rdata = append(rdata, byte(chunkSize))
				rdata = append(rdata, tempData[:chunkSize]...)
				tempData = tempData[chunkSize:]
			}
		}

		response = append(response, byte(len(rdata)>>8), byte(len(rdata)))
		response = append(response, rdata...)

	case aRecordType:
		for i := uint16(0); i < answerCount; i++ {
			response = append(response, byte(dnsPointer>>8), byte(dnsPointer&0xFF))
			response = append(response, 0x00, byte(aRecordType))
			response = append(response, 0x00, byte(dnsClassIN))
			response = append(response, 0x00, 0x00, 0x00, byte(dnsTTLSeconds))
			response = append(response, 0x00, 0x04)

			start := int(i) * recordSize
			end := start + recordSize
			rdata := make([]byte, 4)
			if start < len(data) {
				copyEnd := end
				if copyEnd > len(data) {
					copyEnd = len(data)
				}
				copy(rdata, data[start:copyEnd])
			}
			response = append(response, rdata...)
		}

	case aaaaRecordType:
		for i := uint16(0); i < answerCount; i++ {
			response = append(response, byte(dnsPointer>>8), byte(dnsPointer&0xFF))
			response = append(response, 0x00, byte(aaaaRecordType))
			response = append(response, 0x00, byte(dnsClassIN))
			response = append(response, 0x00, 0x00, 0x00, byte(dnsTTLSeconds))
			response = append(response, 0x00, 0x10)

			start := int(i) * recordSize
			end := start + recordSize
			rdata := make([]byte, 16)
			if start < len(data) {
				copyEnd := end
				if copyEnd > len(data) {
					copyEnd = len(data)
				}
				copy(rdata, data[start:copyEnd])
			}
			response = append(response, rdata...)
		}

	default:
		response = append(response, byte(dnsPointer>>8), byte(dnsPointer&0xFF))
		response = append(response, byte(queryType>>8), byte(queryType&0xFF))
		response = append(response, 0x00, byte(dnsClassIN))
		response = append(response, 0x00, 0x00, 0x00, byte(dnsTTLSeconds))
		response = append(response, 0x00, 0x00)
	}

	if _, err := conn.WriteToUDP(response, addr); err != nil {
		slog.Error("dns redirector: incoming request failed, could not write DNS response", "destination", addr.String(), "error", err)
	}
}

func (r *Redirector) sendErrorResponse(conn *net.UDPConn, addr *net.UDPAddr, transactionID uint16) {
	response := make([]byte, dnsHeaderSize)
	binary.BigEndian.PutUint16(response[0:2], transactionID)
	response[2] = byte(dnsErrorFlags >> 8)
	response[3] = byte(dnsErrorFlags & 0xFF)

	if _, err := conn.WriteToUDP(response, addr); err != nil {
		slog.Error("dns redirector: incoming request failed, could not write DNS error response", "destination", addr.String(), "error", err)
	}
}
