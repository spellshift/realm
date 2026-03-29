package dns

import (
	"encoding/base32"
	"hash/crc32"
	"net"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/proto"
	"realm.pub/tavern/internal/c2/convpb"
)

func newTestRedirector() *Redirector {
	return &Redirector{}
}

// TestParseListenAddr tests the ParseListenAddr function
func TestParseListenAddr(t *testing.T) {
	tests := []struct {
		name            string
		input           string
		expectedAddr    string
		expectedDomains []string
		expectError     bool
	}{
		{
			name:            "default port with multiple domains",
			input:           "0.0.0.0?domain=dnsc2.realm.pub&domain=foo.bar",
			expectedAddr:    "0.0.0.0:53",
			expectedDomains: []string{"dnsc2.realm.pub", "foo.bar"},
		},
		{
			name:            "custom port with single domain",
			input:           "127.0.0.1:8053?domain=dnsc2.realm.pub",
			expectedAddr:    "127.0.0.1:8053",
			expectedDomains: []string{"dnsc2.realm.pub"},
		},
		{
			name:            "no query params",
			input:           "0.0.0.0:5353",
			expectedAddr:    "0.0.0.0:5353",
			expectedDomains: nil,
		},
		{
			name:            "empty domain value",
			input:           "0.0.0.0?domain=",
			expectedAddr:    "0.0.0.0:53",
			expectedDomains: nil,
		},
		{
			name:            "mixed valid and empty domains",
			input:           "0.0.0.0?domain=valid.com&domain=&domain=also.valid",
			expectedAddr:    "0.0.0.0:53",
			expectedDomains: []string{"valid.com", "also.valid"},
		},
		{
			name:        "malformed URL encoding",
			input:       "0.0.0.0?domain=%ZZ",
			expectError: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			addr, domains, err := ParseListenAddr(tt.input)

			if tt.expectError {
				assert.Error(t, err)
				return
			}

			require.NoError(t, err)
			assert.Equal(t, tt.expectedAddr, addr)
			assert.ElementsMatch(t, tt.expectedDomains, domains)
		})
	}
}

// TestExtractSubdomain tests subdomain extraction from full domain names
func TestExtractSubdomain(t *testing.T) {
	r := newTestRedirector()
	r.baseDomains = []string{"dnsc2.realm.pub", "foo.bar.com"}

	tests := []struct {
		name           string
		domain         string
		expectedSubdom string
		expectError    bool
	}{
		{
			name:           "simple subdomain",
			domain:         "test.dnsc2.realm.pub",
			expectedSubdom: "test",
		},
		{
			name:           "multi-label subdomain",
			domain:         "a.b.c.dnsc2.realm.pub",
			expectedSubdom: "a.b.c",
		},
		{
			name:           "subdomain with longer base domain",
			domain:         "test.foo.bar.com",
			expectedSubdom: "test",
		},
		{
			name:        "no matching base domain",
			domain:      "test.unknown.com",
			expectError: true,
		},
		{
			name:        "only base domain (no subdomain)",
			domain:      "dnsc2.realm.pub",
			expectError: true,
		},
		{
			name:           "case insensitive match",
			domain:         "test.DNSC2.REALM.PUB",
			expectedSubdom: "test",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			subdomain, err := r.extractSubdomain(tt.domain)

			if tt.expectError {
				assert.Error(t, err)
				return
			}

			require.NoError(t, err)
			assert.Equal(t, tt.expectedSubdom, subdomain)
		})
	}
}

// TestDecodePacket tests Base32 decoding and protobuf unmarshaling
func TestDecodePacket(t *testing.T) {
	r := newTestRedirector()

	t.Run("valid INIT packet", func(t *testing.T) {
		packet := &convpb.ConvPacket{
			Type:           convpb.PacketType_PACKET_TYPE_INIT,
			Sequence:       0,
			ConversationId: "test1234",
			Data:           []byte{0x01, 0x02, 0x03},
		}
		packetBytes, err := proto.Marshal(packet)
		require.NoError(t, err)

		encoded := base32.StdEncoding.WithPadding(base32.NoPadding).EncodeToString(packetBytes)

		decoded, err := r.decodePacket(encoded)
		require.NoError(t, err)
		assert.Equal(t, convpb.PacketType_PACKET_TYPE_INIT, decoded.Type)
		assert.Equal(t, "test1234", decoded.ConversationId)
		assert.Equal(t, []byte{0x01, 0x02, 0x03}, decoded.Data)
	})

	t.Run("valid DATA packet with CRC", func(t *testing.T) {
		data := []byte{0xDE, 0xAD, 0xBE, 0xEF}
		packet := &convpb.ConvPacket{
			Type:           convpb.PacketType_PACKET_TYPE_DATA,
			Sequence:       1,
			ConversationId: "test5678",
			Data:           data,
			Crc32:          crc32.ChecksumIEEE(data),
		}
		packetBytes, err := proto.Marshal(packet)
		require.NoError(t, err)

		encoded := base32.StdEncoding.WithPadding(base32.NoPadding).EncodeToString(packetBytes)

		decoded, err := r.decodePacket(encoded)
		require.NoError(t, err)
		assert.Equal(t, convpb.PacketType_PACKET_TYPE_DATA, decoded.Type)
		assert.Equal(t, data, decoded.Data)
	})

	t.Run("DATA packet with invalid CRC", func(t *testing.T) {
		data := []byte{0xDE, 0xAD, 0xBE, 0xEF}
		packet := &convpb.ConvPacket{
			Type:           convpb.PacketType_PACKET_TYPE_DATA,
			Sequence:       1,
			ConversationId: "test5678",
			Data:           data,
			Crc32:          0xDEADBEEF, // Wrong CRC
		}
		packetBytes, err := proto.Marshal(packet)
		require.NoError(t, err)

		encoded := base32.StdEncoding.WithPadding(base32.NoPadding).EncodeToString(packetBytes)

		_, err = r.decodePacket(encoded)
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "CRC mismatch")
	})

	t.Run("invalid Base32", func(t *testing.T) {
		_, err := r.decodePacket("!!!invalid!!!")
		assert.Error(t, err)
	})

	t.Run("invalid protobuf", func(t *testing.T) {
		// Valid Base32 but not valid protobuf
		encoded := base32.StdEncoding.WithPadding(base32.NoPadding).EncodeToString([]byte{0xFF, 0xFF, 0xFF})
		_, err := r.decodePacket(encoded)
		assert.Error(t, err)
	})

	t.Run("packet with labels (dots)", func(t *testing.T) {
		packet := &convpb.ConvPacket{
			Type:           convpb.PacketType_PACKET_TYPE_INIT,
			ConversationId: "test1234",
		}
		packetBytes, err := proto.Marshal(packet)
		require.NoError(t, err)

		encoded := base32.StdEncoding.WithPadding(base32.NoPadding).EncodeToString(packetBytes)
		// Split into labels (simulating DNS format)
		withDots := encoded[:4] + "." + encoded[4:]

		decoded, err := r.decodePacket(withDots)
		require.NoError(t, err)
		assert.Equal(t, "test1234", decoded.ConversationId)
	})
}

// TestParseDomainNameAndType tests DNS query parsing
func TestParseDomainNameAndType(t *testing.T) {
	r := newTestRedirector()

	tests := []struct {
		name         string
		query        []byte
		expectDomain string
		expectType   uint16
		expectError  bool
	}{
		{
			name: "valid TXT query",
			query: func() []byte {
				q := []byte{
					5, 'd', 'n', 's', 'c', '2', // "dnsc2"
					5, 'r', 'e', 'a', 'l', 'm', // "realm"
					3, 'p', 'u', 'b', // "pub"
					0,     // null terminator
					0, 16, // Type: TXT
					0, 1, // Class: IN
				}
				return q
			}(),
			expectDomain: "dnsc2.realm.pub",
			expectType:   16,
		},
		{
			name: "valid A query",
			query: func() []byte {
				q := []byte{
					4, 't', 'e', 's', 't', // "test"
					5, 'd', 'n', 's', 'c', '2', // "dnsc2"
					5, 'r', 'e', 'a', 'l', 'm', // "realm"
					3, 'p', 'u', 'b', // "pub"
					0,    // null terminator
					0, 1, // Type: A
					0, 1, // Class: IN
				}
				return q
			}(),
			expectDomain: "test.dnsc2.realm.pub",
			expectType:   1,
		},
		{
			name: "valid AAAA query",
			query: func() []byte {
				q := []byte{
					3, 'w', 'w', 'w', // "www"
					4, 't', 'e', 's', 't', // "test"
					3, 'c', 'o', 'm', // "com"
					0,     // null terminator
					0, 28, // Type: AAAA
					0, 1, // Class: IN
				}
				return q
			}(),
			expectDomain: "www.test.com",
			expectType:   28,
		},
		{
			name:        "truncated query",
			query:       []byte{7, 'e', 'x', 'a'}, // Incomplete
			expectError: true,
		},
		{
			name:        "query too short for type",
			query:       []byte{4, 't', 'e', 's', 't', 0}, // Missing type/class
			expectError: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			domain, queryType, err := r.parseDomainNameAndType(tt.query)

			if tt.expectError {
				assert.Error(t, err)
				return
			}

			require.NoError(t, err)
			assert.Equal(t, tt.expectDomain, domain)
			assert.Equal(t, tt.expectType, queryType)
		})
	}
}

// TestBuildDNSResponse tests DNS response packet construction
func TestBuildDNSResponse(t *testing.T) {
	r := newTestRedirector()

	// Create a mock UDP connection for testing
	serverAddr, err := net.ResolveUDPAddr("udp", "127.0.0.1:0")
	require.NoError(t, err)
	serverConn, err := net.ListenUDP("udp", serverAddr)
	require.NoError(t, err)
	defer serverConn.Close()

	clientAddr, err := net.ResolveUDPAddr("udp", "127.0.0.1:0")
	require.NoError(t, err)
	clientConn, err := net.ListenUDP("udp", clientAddr)
	require.NoError(t, err)
	defer clientConn.Close()

	t.Run("TXT record response", func(t *testing.T) {
		r.sendDNSResponse(serverConn, clientConn.LocalAddr().(*net.UDPAddr), 0x1234, "test.dnsc2.realm.pub", txtRecordType, []byte("hello"))

		buf := make([]byte, 512)
		clientConn.SetReadDeadline(time.Now().Add(time.Second))
		n, _, err := clientConn.ReadFromUDP(buf)
		require.NoError(t, err)

		// Verify transaction ID
		assert.Equal(t, uint16(0x1234), uint16(buf[0])<<8|uint16(buf[1]))
		// Verify it's a response (QR bit set)
		assert.True(t, buf[2]&0x80 != 0)
		// Verify answer count is 1
		assert.Equal(t, uint16(1), uint16(buf[6])<<8|uint16(buf[7]))

		// Response should contain data
		assert.Greater(t, n, 12)
	})
}
