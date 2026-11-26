package portals

import (
	"fmt"
	"time"

	"google.golang.org/protobuf/proto"
	"realm.pub/tavern/portals/portalpb"
	"realm.pub/tavern/portals/tracepb"
)

func AddTraceEvent(mote *portalpb.Mote, kind tracepb.TraceEventKind) (*portalpb.Mote, error) {
	bm := mote.GetBytes()
	if bm == nil || bm.Kind != portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_TRACE {
		return mote, nil
	}

	var traceData tracepb.TraceData
	if err := proto.Unmarshal(bm.Data, &traceData); err != nil {
		return mote, fmt.Errorf("failed to unmarshal trace data: %w", err)
	}

	traceData.Events = append(traceData.Events, &tracepb.TraceEvent{
		Kind:            kind,
		TimestampMicros: time.Now().UTC().UnixMicro(),
	})

	newData, err := proto.Marshal(&traceData)
	if err != nil {
		return mote, fmt.Errorf("failed to marshal trace data: %w", err)
	}

	cloned := proto.Clone(mote).(*portalpb.Mote)
	cloned.GetBytes().Data = newData
	return cloned, nil
}
