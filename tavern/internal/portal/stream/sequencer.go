package stream

import (
	"sync/atomic"

	"realm.pub/tavern/internal/portal/portalpb"
)

type PayloadSequencer struct {
	nextSeqID atomic.Uint64
}

func NewPayloadSequencer() *PayloadSequencer {
	return &PayloadSequencer{}
}

func (s *PayloadSequencer) GetSeqID() uint64 {
	return s.nextSeqID.Add(1) - 1
}

func (s *PayloadSequencer) NewBytesPayload(data []byte, kind portalpb.BytesMessageKind) *portalpb.Payload {
	return &portalpb.Payload{
		SeqId: s.GetSeqID(),
		Payload: &portalpb.Payload_Bytes{
			Bytes: &portalpb.BytesMessage{
				Data: data,
				Kind: kind,
			},
		},
	}
}

func (s *PayloadSequencer) NewTCPPayload(data []byte, dstAddr string, dstPort uint32, srcID string) *portalpb.Payload {
	return &portalpb.Payload{
		SeqId: s.GetSeqID(),
		Payload: &portalpb.Payload_Tcp{
			Tcp: &portalpb.TCPMessage{
				Data:    data,
				DstAddr: dstAddr,
				DstPort: dstPort,
				SrcId:   srcID,
			},
		},
	}
}

func (s *PayloadSequencer) NewUDPPayload(data []byte, dstAddr string, dstPort uint32, srcID string) *portalpb.Payload {
	return &portalpb.Payload{
		SeqId: s.GetSeqID(),
		Payload: &portalpb.Payload_Udp{
			Udp: &portalpb.UDPMessage{
				Data:    data,
				DstAddr: dstAddr,
				DstPort: dstPort,
				SrcId:   srcID,
			},
		},
	}
}
