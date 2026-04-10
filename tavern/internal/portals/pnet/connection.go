package pnet

import (
	"fmt"
	"io"
	"net"
	"strconv"
	"time"

	"realm.pub/tavern/portals/portalpb"
	"realm.pub/tavern/portals/stream"
)

// Connection implements net.Conn over portal motes.
type Connection struct {
	network string
	dstAddr string
	dstPort uint32

	reader *stream.OrderedReader
	writer *stream.OrderedWriter

	readBuf []byte
}

// Dial creates a new portal network connection.
func Dial(network, address, streamID string, sender stream.SenderFunc, receiver stream.ReceiverFunc) (net.Conn, error) {
	if network != "tcp" && network != "udp" {
		return nil, fmt.Errorf("unsupported network %q", network)
	}

	host, portStr, err := net.SplitHostPort(address)
	if err != nil {
		return nil, fmt.Errorf("invalid address: %w", err)
	}

	port, err := strconv.ParseUint(portStr, 10, 32)
	if err != nil {
		return nil, fmt.Errorf("invalid port: %w", err)
	}

	return &Connection{
		network: network,
		dstAddr: host,
		dstPort: uint32(port),
		reader:  stream.NewOrderedReader(receiver),
		writer:  stream.NewOrderedWriter(streamID, sender),
	}, nil
}

// Read reads data from the connection.
func (c *Connection) Read(b []byte) (n int, err error) {
	if len(c.readBuf) > 0 {
		n = copy(b, c.readBuf)
		c.readBuf = c.readBuf[n:]
		return n, nil
	}

	for {
		mote, err := c.reader.Read()
		if err != nil {
			return 0, err
		}

		var payloadData []byte
		switch p := mote.Payload.(type) {
		case *portalpb.Mote_Bytes:
			if p.Bytes.Kind == portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE {
				return 0, io.EOF
			}
			payloadData = p.Bytes.Data
		case *portalpb.Mote_Tcp:
			payloadData = p.Tcp.Data
		case *portalpb.Mote_Udp:
			payloadData = p.Udp.Data
		default:
			// ignore other payloads
			continue
		}

		if len(payloadData) == 0 {
			continue // keep waiting for actual data
		}

		n = copy(b, payloadData)
		if n < len(payloadData) {
			c.readBuf = payloadData[n:]
		}
		return n, nil
	}
}

// Write writes data to the connection.
func (c *Connection) Write(b []byte) (n int, err error) {
	if len(b) == 0 {
		return 0, nil
	}

	if c.network == "tcp" {
		err = c.writer.WriteTCP(b, c.dstAddr, c.dstPort)
	} else if c.network == "udp" {
		err = c.writer.WriteUDP(b, c.dstAddr, c.dstPort)
	} else {
		return 0, fmt.Errorf("unsupported network %q", c.network)
	}

	if err != nil {
		return 0, err
	}
	return len(b), nil
}

// Close closes the connection.
func (c *Connection) Close() error {
	// Signal close
	err := c.writer.WriteBytes(nil, portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE)
	c.reader.Close()
	return err
}

// LocalAddr returns the local network address.
func (c *Connection) LocalAddr() net.Addr {
	return &dummyAddr{network: c.network, address: "localhost:0"}
}

// RemoteAddr returns the remote network address.
func (c *Connection) RemoteAddr() net.Addr {
	return &dummyAddr{
		network: c.network,
		address: net.JoinHostPort(c.dstAddr, strconv.Itoa(int(c.dstPort))),
	}
}

// SetDeadline sets the read and write deadlines associated with the connection.
func (c *Connection) SetDeadline(t time.Time) error {
	if err := c.SetReadDeadline(t); err != nil {
		return err
	}
	return c.SetWriteDeadline(t)
}

// SetReadDeadline sets the deadline for future Read calls.
func (c *Connection) SetReadDeadline(t time.Time) error {
	c.reader.SetReadDeadline(t)
	return nil
}

// SetWriteDeadline sets the deadline for future Write calls.
func (c *Connection) SetWriteDeadline(t time.Time) error {
	c.writer.SetWriteDeadline(t)
	return nil
}

type dummyAddr struct {
	network string
	address string
}

func (a *dummyAddr) Network() string { return a.network }
func (a *dummyAddr) String() string  { return a.address }
