package stream

import (
	"realm.pub/tavern/portals/portalpb"
)

// SenderFunc is a callback that writes a Mote to a destination.
// This allows OrderedWriter to wrap any gRPC stream method.
type SenderFunc func(*portalpb.Mote) error

// OrderedWriter uses a payloadSequencer to create motes that are then written to a destination.
type OrderedWriter struct {
	sequencer *payloadSequencer
	sender    SenderFunc
}

// NewOrderedWriter creates a new OrderedWriter.
func NewOrderedWriter(streamID string, sender SenderFunc) *OrderedWriter {
	return &OrderedWriter{
		sequencer: newPayloadSequencer(streamID),
		sender:    sender,
	}
}

// WriteBytes creates and writes a BytesMote.
func (w *OrderedWriter) WriteBytes(data []byte, kind portalpb.BytesPayloadKind) error {
	mote := w.sequencer.NewBytesMote(data, kind)
	return w.sender(mote)
}

// WriteTCP creates and writes a TCPMote.
func (w *OrderedWriter) WriteTCP(data []byte, dstAddr string, dstPort uint32) error {
	mote := w.sequencer.NewTCPMote(data, dstAddr, dstPort)
	return w.sender(mote)
}

// WriteUDP creates and writes a UDPMote.
func (w *OrderedWriter) WriteUDP(data []byte, dstAddr string, dstPort uint32) error {
	mote := w.sequencer.NewUDPMote(data, dstAddr, dstPort)
	return w.sender(mote)
}

// WriteShell creates and writes a ShellMote.
func (w *OrderedWriter) WriteShell(data []byte) error {
	mote := w.sequencer.NewShellMote(data)
	return w.sender(mote)
}

// WriteRepl creates and writes a ReplMote.
func (w *OrderedWriter) WriteRepl(data []byte) error {
	mote := w.sequencer.NewReplMote(data)
	return w.sender(mote)
}
