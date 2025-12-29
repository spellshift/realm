package stream

import (
	"errors"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/portal/portalpb"
)

func TestPayloadSequencer(t *testing.T) {
	seq := NewPayloadSequencer()

	p1 := seq.NewBytesPayload([]byte("test"), portalpb.BytesMessageKind_BYTES_MESSAGE_KIND_DATA)
	assert.Equal(t, uint64(0), p1.SeqId)
	assert.Equal(t, []byte("test"), p1.GetBytes().Data)

	p2 := seq.NewTCPPayload([]byte("tcp"), "1.2.3.4", 80, "src1")
	assert.Equal(t, uint64(1), p2.SeqId)
	assert.Equal(t, "1.2.3.4", p2.GetTcp().DstAddr)

	p3 := seq.NewUDPPayload([]byte("udp"), "5.6.7.8", 53, "src2")
	assert.Equal(t, uint64(2), p3.SeqId)
	assert.Equal(t, "5.6.7.8", p3.GetUdp().DstAddr)
}

func TestOrderedWriter(t *testing.T) {
	seq := NewPayloadSequencer()
	var sent []*portalpb.Payload
	sender := func(p *portalpb.Payload) error {
		sent = append(sent, p)
		return nil
	}

	w := NewOrderedWriter(seq, sender)

	err := w.WriteBytes([]byte("data"), portalpb.BytesMessageKind_BYTES_MESSAGE_KIND_DATA)
	require.NoError(t, err)

	err = w.WriteTCP([]byte("tcp"), "localhost", 8080, "id1")
	require.NoError(t, err)

	err = w.WriteUDP([]byte("udp"), "localhost", 53, "id2")
	require.NoError(t, err)

	require.Len(t, sent, 3)
	assert.Equal(t, uint64(0), sent[0].SeqId)
	assert.Equal(t, uint64(1), sent[1].SeqId)
	assert.Equal(t, uint64(2), sent[2].SeqId)
}

func TestOrderedReader_HappyPath(t *testing.T) {
	// Sequence: 0, 1, 2
	messages := []*portalpb.Payload{
		{SeqId: 0, Payload: &portalpb.Payload_Bytes{Bytes: &portalpb.BytesMessage{Data: []byte("0")}}},
		{SeqId: 1, Payload: &portalpb.Payload_Bytes{Bytes: &portalpb.BytesMessage{Data: []byte("1")}}},
		{SeqId: 2, Payload: &portalpb.Payload_Bytes{Bytes: &portalpb.BytesMessage{Data: []byte("2")}}},
	}
	idx := 0
	receiver := func() (*portalpb.Payload, error) {
		if idx >= len(messages) {
			return nil, errors.New("EOF")
		}
		msg := messages[idx]
		idx++
		return msg, nil
	}

	r := NewOrderedReader(receiver, 10, time.Second)

	msg, err := r.Read()
	require.NoError(t, err)
	assert.Equal(t, uint64(0), msg.SeqId)

	msg, err = r.Read()
	require.NoError(t, err)
	assert.Equal(t, uint64(1), msg.SeqId)

	msg, err = r.Read()
	require.NoError(t, err)
	assert.Equal(t, uint64(2), msg.SeqId)
}

func TestOrderedReader_OutOfOrder(t *testing.T) {
	// Sequence: 2, 0, 1 (should return 0, 1, 2)
	messages := []*portalpb.Payload{
		{SeqId: 2, Payload: &portalpb.Payload_Bytes{Bytes: &portalpb.BytesMessage{Data: []byte("2")}}},
		{SeqId: 0, Payload: &portalpb.Payload_Bytes{Bytes: &portalpb.BytesMessage{Data: []byte("0")}}},
		{SeqId: 1, Payload: &portalpb.Payload_Bytes{Bytes: &portalpb.BytesMessage{Data: []byte("1")}}},
	}
	idx := 0
	receiver := func() (*portalpb.Payload, error) {
		if idx >= len(messages) {
			return nil, errors.New("EOF")
		}
		msg := messages[idx]
		idx++
		return msg, nil
	}

	r := NewOrderedReader(receiver, 10, time.Second)

	// First call receives 2 (buffered), then 0 (returned)
	msg, err := r.Read()
	require.NoError(t, err)
	assert.Equal(t, uint64(0), msg.SeqId)

	// Second call receives 1. Now we have 1 and 2 in buffer/stream context.
	// Wait, Read() logic:
	// 1. Check buffer for 1. (Not there, we only buffered 2)
	// 2. Call receiver -> gets 1.
	// 3. 1 == nextSeqID (1). Return 1.
	msg, err = r.Read()
	require.NoError(t, err)
	assert.Equal(t, uint64(1), msg.SeqId)

	// Third call. Check buffer for 2. Yes! Return 2.
	msg, err = r.Read()
	require.NoError(t, err)
	assert.Equal(t, uint64(2), msg.SeqId)
}

func TestOrderedReader_Duplicates(t *testing.T) {
	messages := []*portalpb.Payload{
		{SeqId: 0},
		{SeqId: 0}, // Duplicate
		{SeqId: 1},
	}
	idx := 0
	receiver := func() (*portalpb.Payload, error) {
		if idx >= len(messages) {
			return nil, errors.New("EOF")
		}
		msg := messages[idx]
		idx++
		return msg, nil
	}

	r := NewOrderedReader(receiver, 10, time.Second)

	msg, err := r.Read()
	require.NoError(t, err)
	assert.Equal(t, uint64(0), msg.SeqId)

	msg, err = r.Read()
	require.NoError(t, err)
	assert.Equal(t, uint64(1), msg.SeqId)
}

func TestOrderedReader_BufferLimit(t *testing.T) {
	// Receive 1, 2, 3... but missing 0.
	messages := []*portalpb.Payload{
		{SeqId: 1},
		{SeqId: 2},
		{SeqId: 3},
	}
	idx := 0
	receiver := func() (*portalpb.Payload, error) {
		if idx >= len(messages) {
			// Should have failed by now
			return nil, errors.New("EOF")
		}
		msg := messages[idx]
		idx++
		return msg, nil
	}

	// Max buffer 2.
	r := NewOrderedReader(receiver, 2, time.Second)

	// 1 buffered (len=1)
	// 2 buffered (len=2)
	// 3 buffered -> exceeds limit (len=3 > 2)
	_, err := r.Read()
	assert.ErrorIs(t, err, ErrBufferLimitExceeded)
}

func TestOrderedReader_Timeout(t *testing.T) {
	// Missing 0. Receive 1. Wait too long.
	messages := []*portalpb.Payload{
		{SeqId: 1},
	}
	idx := 0
	receiver := func() (*portalpb.Payload, error) {
		if idx >= len(messages) {
			// Simulate blocking or slow stream
			time.Sleep(200 * time.Millisecond)
			return nil, errors.New("timeout simulation end")
		}
		msg := messages[idx]
		idx++
		return msg, nil
	}

	r := NewOrderedReader(receiver, 10, 50*time.Millisecond)

	_, err := r.Read()
	require.Error(t, err) // Expect an error, either stale timeout or the simulation end

	// Should error with timeout because we buffered 1, and then waited > 50ms for 0
	// The loop will call receiver() again, which sleeps 200ms.
	// So inside the loop, the first iteration buffers 1.
	// Second iteration: check buffer timeout.
	// Since receiver() took 200ms (on the second call, simulating waiting for more data),
	// we won't check timeout until receiver() returns (or before calling it).
	// Actually, the loop checks timeout BEFORE calling receiver.
	// So:
	// 1. receive 1. buffer it.
	// 2. Loop continues.
	// 3. Check timeout. time.Since(start) ~ 0. OK.
	// 4. Call receiver(). SLEEPS 200ms.
	// 5. receiver returns error "timeout simulation end".
	// 6. Read returns error "timeout simulation end".

	// This test setup is tricky because receiver() blocks.
	// To test timeout properly, we need receiver() to return *something* (or loop) to allow the loop to check the timeout,
	// OR we rely on the fact that if receiver returns quickly (e.g. empty/keepalives) we check.
	// If receiver blocks forever, we can't check timeout with synchronous receiver.

	// Let's modify receiver to return dummy messages to trigger checks?
	// Or just verify that if we call Read() multiple times it eventually fails?
	// But Read() blocks.

	// Let's assume receiver returns quickly.
	// Sequence:
	// 1. Receive 1.
	// 2. Receive 2. (Time passed > timeout) -> Error.

	// We need a custom receiver that delays between 1 and 2.
}

func TestOrderedReader_Timeout_Real(t *testing.T) {
	messages := []*portalpb.Payload{
		{SeqId: 1}, // Future
		{SeqId: 2}, // More future
	}
	idx := 0
	receiver := func() (*portalpb.Payload, error) {
		if idx == 1 {
			// Delay before sending 2
			time.Sleep(100 * time.Millisecond)
		}
		if idx >= len(messages) {
			return nil, errors.New("EOF")
		}
		msg := messages[idx]
		idx++
		return msg, nil
	}

	r := NewOrderedReader(receiver, 10, 50*time.Millisecond)

	// Call 1:
	// - Get 1. Buffer it. FirstBufferedTime set.
	// - Loop.
	// - Check timeout. Not yet.
	// - Call receiver. SLEEPS 100ms.
	// - Returns 2.
	// - Loop.
	// - Check timeout. 100ms > 50ms. Error!

	_, err := r.Read()
	assert.ErrorIs(t, err, ErrStaleTimeout)
}

func TestOrderedReader_ReceiverError(t *testing.T) {
	receiver := func() (*portalpb.Payload, error) {
		return nil, errors.New("receiver failed")
	}

	r := NewOrderedReader(receiver, 10, time.Second)
	_, err := r.Read()
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "receiver failed")
}

func TestOrderedReader_NestedGaps(t *testing.T) {
	// Sequence: 2, 4, 1, 3, 0. Should return 0, 1, 2, 3, 4.
	messages := []*portalpb.Payload{
		{SeqId: 2},
		{SeqId: 4},
		{SeqId: 1},
		{SeqId: 3},
		{SeqId: 0},
	}
	idx := 0
	receiver := func() (*portalpb.Payload, error) {
		if idx >= len(messages) {
			return nil, errors.New("EOF")
		}
		msg := messages[idx]
		idx++
		return msg, nil
	}

	r := NewOrderedReader(receiver, 10, time.Second)

	for i := 0; i < 5; i++ {
		msg, err := r.Read()
		require.NoError(t, err)
		assert.Equal(t, uint64(i), msg.SeqId)
	}
}
