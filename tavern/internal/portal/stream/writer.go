package stream

import "realm.pub/tavern/internal/portal/portalpb"

type Sender func(*portalpb.Payload) error

type OrderedWriter struct {
	sequencer *PayloadSequencer
	sender    Sender
}

func NewOrderedWriter(sequencer *PayloadSequencer, sender Sender) *OrderedWriter {
	return &OrderedWriter{
		sequencer: sequencer,
		sender:    sender,
	}
}

// WriteBytes creates a Bytes payload and sends it.
func (w *OrderedWriter) WriteBytes(data []byte, kind portalpb.BytesMessageKind) error {
	payload := w.sequencer.NewBytesPayload(data, kind)
	return w.sender(payload)
}

// WriteTCP creates a TCP payload and sends it.
func (w *OrderedWriter) WriteTCP(data []byte, dstAddr string, dstPort uint32, srcID string) error {
	payload := w.sequencer.NewTCPPayload(data, dstAddr, dstPort, srcID)
	return w.sender(payload)
}

// WriteUDP creates a UDP payload and sends it.
func (w *OrderedWriter) WriteUDP(data []byte, dstAddr string, dstPort uint32, srcID string) error {
	payload := w.sequencer.NewUDPPayload(data, dstAddr, dstPort, srcID)
	return w.sender(payload)
}
