package stream

import (
	"sync/atomic"

	"realm.pub/tavern/portals/portalpb"
)

// PayloadSequencer sequences payloads with a stream ID and monotonic sequence ID.
type PayloadSequencer struct {
	nextSeqID atomic.Uint64
	streamID  string
}

// NewPayloadSequencer creates a new PayloadSequencer with the given streamID.
func NewPayloadSequencer(streamID string) *PayloadSequencer {
	return &PayloadSequencer{
		streamID: streamID,
	}
}

// newSeqID returns the current sequence ID and increments it.
func (s *PayloadSequencer) newSeqID() uint64 {
	return s.nextSeqID.Add(1) - 1
}

// NewBytesMote creates a new Mote with a BytesPayload.
func (s *PayloadSequencer) NewBytesMote(data []byte, kind portalpb.BytesPayloadKind) *portalpb.Mote {
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
func (s *PayloadSequencer) NewTCPMote(data []byte, dstAddr string, dstPort uint32) *portalpb.Mote {
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
func (s *PayloadSequencer) NewUDPMote(data []byte, dstAddr string, dstPort uint32) *portalpb.Mote {
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

// NewShellMote creates a new Mote with a ShellPayload.
func (s *PayloadSequencer) NewShellMote(input string, shellID int64, taskID uint64) *portalpb.Mote {
	return &portalpb.Mote{
		StreamId: s.streamID,
		SeqId:    s.newSeqID(),
		Payload: &portalpb.Mote_Shell{
			Shell: &portalpb.ShellPayload{
				Input:   input,
				ShellId: shellID,
				TaskId:  taskID,
			},
		},
	}
}
