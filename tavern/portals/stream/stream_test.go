package stream

import (
	"errors"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/portals/portalpb"
)

func TestSequencer(t *testing.T) {
	streamID := "test-stream-id"
	seq := newPayloadSequencer(streamID)
	assert.Equal(t, streamID, seq.streamID)
	assert.Equal(t, uint64(0), seq.nextSeqID.Load())

	// Test newSeqID
	id1 := seq.newSeqID()
	assert.Equal(t, uint64(0), id1)
	id2 := seq.newSeqID()
	assert.Equal(t, uint64(1), id2)

	// Test Mote creation
	mote := seq.NewBytesMote([]byte("test"), portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_DATA)
	assert.Equal(t, seq.streamID, mote.StreamId)
	assert.Equal(t, uint64(2), mote.SeqId)
	assert.IsType(t, &portalpb.Mote_Bytes{}, mote.Payload)
}

func TestOrderedWriter(t *testing.T) {
	streamID := "test-stream-id"
	var sentMote *portalpb.Mote
	sender := func(m *portalpb.Mote) error {
		sentMote = m
		return nil
	}

	w := NewOrderedWriter(streamID, sender)

	err := w.WriteBytes([]byte("hello"), portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_DATA)
	require.NoError(t, err)
	require.NotNil(t, sentMote)
	assert.Equal(t, streamID, sentMote.StreamId)
	assert.Equal(t, uint64(0), sentMote.SeqId)
	assert.Equal(t, []byte("hello"), sentMote.GetBytes().Data)

	err = w.WriteTCP([]byte("tcp"), "1.2.3.4", 80)
	require.NoError(t, err)
	assert.Equal(t, uint64(1), sentMote.SeqId)
	assert.Equal(t, "1.2.3.4", sentMote.GetTcp().DstAddr)
}

func TestOrderedReader_Ordered(t *testing.T) {
	motes := []*portalpb.Mote{
		{SeqId: 0, StreamId: "s1"},
		{SeqId: 1, StreamId: "s1"},
		{SeqId: 2, StreamId: "s1"},
	}
	idx := 0
	receiver := func() (*portalpb.Mote, error) {
		if idx >= len(motes) {
			return nil, errors.New("EOF")
		}
		m := motes[idx]
		idx++
		return m, nil
	}

	r := NewOrderedReader(receiver, 10, time.Second)

	for i := 0; i < 3; i++ {
		m, err := r.Read()
		require.NoError(t, err)
		assert.Equal(t, uint64(i), m.SeqId)
	}
}

func TestOrderedReader_OutOfOrder(t *testing.T) {
	// Send 2, 0, 1
	motes := []*portalpb.Mote{
		{SeqId: 2, StreamId: "s1"},
		{SeqId: 0, StreamId: "s1"},
		{SeqId: 1, StreamId: "s1"},
	}
	idx := 0
	receiver := func() (*portalpb.Mote, error) {
		if idx >= len(motes) {
			// Instead of erroring immediately, block or return nil to simulate waiting?
			// The reader loop will continue if we don't return error.
			// But here we expect to consume all.
			return nil, errors.New("EOF")
		}
		m := motes[idx]
		idx++
		return m, nil
	}

	r := NewOrderedReader(receiver, 10, time.Second)

	// First Read should get 0 (which arrives 2nd)
	m0, err := r.Read()
	require.NoError(t, err)
	assert.Equal(t, uint64(0), m0.SeqId)

	// Second Read should get 1 (which arrives 3rd)
	m1, err := r.Read()
	require.NoError(t, err)
	assert.Equal(t, uint64(1), m1.SeqId)

	// Third Read should get 2 (which arrived 1st and was buffered)
	m2, err := r.Read()
	require.NoError(t, err)
	assert.Equal(t, uint64(2), m2.SeqId)
}

func TestOrderedReader_StaleTimeout(t *testing.T) {
	// Send 1 (gap, missing 0) and wait
	receiver := func() (*portalpb.Mote, error) {
		time.Sleep(10 * time.Millisecond)
		return &portalpb.Mote{SeqId: 1, StreamId: "s1"}, nil
	}

	// Timeout very short
	r := NewOrderedReader(receiver, 10, 50*time.Millisecond)

	// First read will receive 1, buffer it, then loop.
	// Receiver sleeps 10ms.
	// It will keep receiving 1 (duplicate) or we need to simulate a stream of future packets?
	// The receiver above returns 1 every time.
	// So:
	// 1. Read() calls receiver -> gets 1. SeqID > 0. Buffers 1. Loop.
	// 2. Read() calls receiver -> gets 1. SeqID > 0. Ignored (duplicate logic?).
	//    Wait, duplicates: "mote.SeqId > r.nextSeqID".
	//    If duplicate in buffer, we overwrite or ignore.
	//    "if _, exists := r.buffer[mote.SeqId]; !exists { r.buffer[...] = ... }"
	//    So it ignores duplicates in buffer.
	//    Loop continues.
	// 3. Eventually time.Since(firstBufferedAt) > 50ms.
	//    Should error.

	_, err := r.Read()
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "timeout")
}

func TestOrderedReader_BufferLimit(t *testing.T) {
	// Send 1, 2, 3... buffer limit 2. Missing 0.
	idx := 1
	receiver := func() (*portalpb.Mote, error) {
		m := &portalpb.Mote{SeqId: uint64(idx), StreamId: "s1"}
		idx++
		return m, nil
	}

	r := NewOrderedReader(receiver, 2, time.Second)

	_, err := r.Read()
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "buffer limit exceeded")
}

func TestOrderedReader_DuplicateHandling(t *testing.T) {
	// Send 0, 0, 1
	motes := []*portalpb.Mote{
		{SeqId: 0, StreamId: "s1"},
		{SeqId: 0, StreamId: "s1"}, // Duplicate
		{SeqId: 1, StreamId: "s1"},
	}
	idx := 0
	receiver := func() (*portalpb.Mote, error) {
		if idx >= len(motes) {
			return nil, errors.New("EOF")
		}
		m := motes[idx]
		idx++
		return m, nil
	}

	r := NewOrderedReader(receiver, 10, time.Second)

	m0, err := r.Read()
	require.NoError(t, err)
	assert.Equal(t, uint64(0), m0.SeqId)

	m1, err := r.Read()
	require.NoError(t, err)
	assert.Equal(t, uint64(1), m1.SeqId)
}
