package stream

import (
	"sync/atomic"

	"realm.pub/tavern/portals/portalpb"
)

// payloadSequencer sequences payloads with a stream ID and monotonic sequence ID.
type payloadSequencer struct {
	nextSeqID atomic.Uint64
	streamID  string
}

// newPayloadSequencer creates a new payloadSequencer with the given streamID.
func newPayloadSequencer(streamID string) *payloadSequencer {
	return &payloadSequencer{
		streamID: streamID,
	}
}

// newSeqID returns the current sequence ID and increments it.
func (s *payloadSequencer) newSeqID() uint64 {
	return s.nextSeqID.Add(1) - 1
}

// NewBytesMote creates a new Mote with a BytesPayload.
func (s *payloadSequencer) NewBytesMote(data []byte, kind portalpb.BytesPayloadKind) *portalpb.Mote {
	return &portalpb.Mote{
		StreamId: s.streamID,
		SeqId:    s.newSeqID(),
		Payload: &portalpb.Mote_Bytes{
			Bytes: &portalpb.BytesPayload{
				Data: data,
				Kind: kind,
			},
		},
	}
}

// NewTCPMote creates a new Mote with a TCPPayload.
func (s *payloadSequencer) NewTCPMote(data []byte, dstAddr string, dstPort uint32) *portalpb.Mote {
	return &portalpb.Mote{
		StreamId: s.streamID,
		SeqId:    s.newSeqID(),
		Payload: &portalpb.Mote_Tcp{
			Tcp: &portalpb.TCPPayload{
				Data:    data,
				DstAddr: dstAddr,
				DstPort: dstPort,
			},
		},
	}
}

// NewUDPMote creates a new Mote with a UDPPayload.
func (s *payloadSequencer) NewUDPMote(data []byte, dstAddr string, dstPort uint32) *portalpb.Mote {
	return &portalpb.Mote{
		StreamId: s.streamID,
		SeqId:    s.newSeqID(),
		Payload: &portalpb.Mote_Udp{
			Udp: &portalpb.UDPPayload{
				Data:    data,
				DstAddr: dstAddr,
				DstPort: dstPort,
			},
		},
	}
}
