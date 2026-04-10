package pnet

import (
	"io"
	"os"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/portals/portalpb"
)

func TestConnection_ReadWriteTCP(t *testing.T) {
	moteCh := make(chan *portalpb.Mote, 10)

	sender := func(m *portalpb.Mote) error {
		moteCh <- m
		return nil
	}

	receiver := func() (*portalpb.Mote, error) {
		m, ok := <-moteCh
		if !ok {
			return nil, io.EOF
		}
		return m, nil
	}

	conn, err := Dial("tcp", "192.168.1.1:80", "test-stream", sender, receiver)
	require.NoError(t, err)
	defer conn.Close()

	assert.Equal(t, "tcp", conn.LocalAddr().Network())
	assert.Equal(t, "localhost:0", conn.LocalAddr().String())
	assert.Equal(t, "tcp", conn.RemoteAddr().Network())
	assert.Equal(t, "192.168.1.1:80", conn.RemoteAddr().String())

	// Test Write
	n, err := conn.Write([]byte("hello world"))
	require.NoError(t, err)
	assert.Equal(t, 11, n)

	// Test Read
	buf := make([]byte, 5)
	n, err = conn.Read(buf)
	require.NoError(t, err)
	assert.Equal(t, 5, n)
	assert.Equal(t, "hello", string(buf[:n]))

	// Test Read remaining
	buf2 := make([]byte, 10)
	n, err = conn.Read(buf2)
	require.NoError(t, err)
	assert.Equal(t, 6, n)
	assert.Equal(t, " world", string(buf2[:n]))
}

func TestConnection_ReadWriteUDP(t *testing.T) {
	moteCh := make(chan *portalpb.Mote, 10)

	sender := func(m *portalpb.Mote) error {
		moteCh <- m
		return nil
	}

	receiver := func() (*portalpb.Mote, error) {
		m, ok := <-moteCh
		if !ok {
			return nil, io.EOF
		}
		return m, nil
	}

	conn, err := Dial("udp", "10.0.0.1:53", "test-stream", sender, receiver)
	require.NoError(t, err)
	defer conn.Close()

	// Test Write
	n, err := conn.Write([]byte("dns query"))
	require.NoError(t, err)
	assert.Equal(t, 9, n)

	// Test Read
	buf := make([]byte, 64)
	n, err = conn.Read(buf)
	require.NoError(t, err)
	assert.Equal(t, 9, n)
	assert.Equal(t, "dns query", string(buf[:n]))
}

func TestConnection_CloseSignalsEOF(t *testing.T) {
	moteCh := make(chan *portalpb.Mote, 10)

	sender := func(m *portalpb.Mote) error {
		moteCh <- m
		return nil
	}

	receiver := func() (*portalpb.Mote, error) {
		m, ok := <-moteCh
		if !ok {
			return nil, io.EOF
		}
		return m, nil
	}

	conn, err := Dial("tcp", "1.1.1.1:443", "test-stream", sender, receiver)
	require.NoError(t, err)

	// simulate a remote close
	conn.(*Connection).writer.WriteBytes(nil, portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE)

	// Reading should hit the close mote
	buf := make([]byte, 64)
	_, err = conn.Read(buf)
	assert.ErrorIs(t, err, io.EOF)
}

func TestConnection_Deadlines(t *testing.T) {
	moteCh := make(chan *portalpb.Mote, 10)

	sender := func(m *portalpb.Mote) error {
		time.Sleep(10 * time.Millisecond) // slight delay
		moteCh <- m
		return nil
	}

	receiver := func() (*portalpb.Mote, error) {
		m := <-moteCh
		return m, nil
	}

	conn, err := Dial("tcp", "1.1.1.1:443", "test-stream", sender, receiver)
	require.NoError(t, err)
	defer conn.Close()

	// Set an immediate read deadline
	err = conn.SetReadDeadline(time.Now().Add(-1 * time.Second))
	require.NoError(t, err)

	buf := make([]byte, 64)
	_, err = conn.Read(buf)
	assert.ErrorIs(t, err, os.ErrDeadlineExceeded)

	// Reset deadline
	err = conn.SetReadDeadline(time.Time{})
	require.NoError(t, err)

	// Set an immediate write deadline
	err = conn.SetWriteDeadline(time.Now().Add(-1 * time.Second))
	require.NoError(t, err)

	_, err = conn.Write([]byte("too late"))
	assert.ErrorIs(t, err, os.ErrDeadlineExceeded)
}
