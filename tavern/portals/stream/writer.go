package stream

import (
	"os"
	"sync"
	"time"

	"realm.pub/tavern/portals/portalpb"
)

// SenderFunc is a callback that writes a Mote to a destination.
// This allows OrderedWriter to wrap any gRPC stream method.
type SenderFunc func(*portalpb.Mote) error

// OrderedWriter uses a payloadSequencer to create motes that are then written to a destination.
type OrderedWriter struct {
	sequencer     *payloadSequencer
	sender        SenderFunc
	deadlineMu    sync.RWMutex
	writeDeadline time.Time
}

// NewOrderedWriter creates a new OrderedWriter.
func NewOrderedWriter(streamID string, sender SenderFunc) *OrderedWriter {
	return &OrderedWriter{
		sequencer: newPayloadSequencer(streamID),
		sender:    sender,
	}
}

// SetWriteDeadline sets the deadline for future Write calls.
func (w *OrderedWriter) SetWriteDeadline(t time.Time) {
	w.deadlineMu.Lock()
	w.writeDeadline = t
	w.deadlineMu.Unlock()
}

func (w *OrderedWriter) checkDeadline() error {
	w.deadlineMu.RLock()
	deadline := w.writeDeadline
	w.deadlineMu.RUnlock()

	if !deadline.IsZero() && time.Now().After(deadline) {
		return os.ErrDeadlineExceeded
	}
	return nil
}

// WriteBytes creates and writes a BytesMote.
func (w *OrderedWriter) WriteBytes(data []byte, kind portalpb.BytesPayloadKind) error {
	if err := w.checkDeadline(); err != nil {
		return err
	}
	mote := w.sequencer.NewBytesMote(data, kind)
	return w.sender(mote)
}

// WriteTCP creates and writes a TCPMote.
func (w *OrderedWriter) WriteTCP(data []byte, dstAddr string, dstPort uint32) error {
	if err := w.checkDeadline(); err != nil {
		return err
	}
	mote := w.sequencer.NewTCPMote(data, dstAddr, dstPort)
	return w.sender(mote)
}

// WriteUDP creates and writes a UDPMote.
func (w *OrderedWriter) WriteUDP(data []byte, dstAddr string, dstPort uint32) error {
	if err := w.checkDeadline(); err != nil {
		return err
	}
	mote := w.sequencer.NewUDPMote(data, dstAddr, dstPort)
	return w.sender(mote)
}

// WriteSSH creates and writes an SSHMote.
func (w *OrderedWriter) WriteSSH(sshPayload *portalpb.SSHPayload) error {
	if err := w.checkDeadline(); err != nil {
		return err
	}
	mote := w.sequencer.NewSSHMote(sshPayload)
	return w.sender(mote)
}
