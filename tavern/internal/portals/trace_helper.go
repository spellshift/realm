package portals

import (
	"fmt"
	"time"

	"google.golang.org/protobuf/proto"
	"realm.pub/tavern/portals/portalpb"
	"realm.pub/tavern/portals/tracepb"
)

func AddTraceEvent(mote *portalpb.Mote, kind tracepb.TraceEventKind, serverID string) error {
	bm := mote.GetBytes()
	if bm == nil || bm.Kind != portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_TRACE {
		return nil
	}

	var traceData tracepb.TraceData
	if err := proto.Unmarshal(bm.Data, &traceData); err != nil {
		return fmt.Errorf("failed to unmarshal trace data: %w", err)
	}

	traceData.Events = append(traceData.Events, &tracepb.TraceEvent{
		Kind:            kind,
		TimestampMicros: time.Now().UTC().UnixMicro(),
		ServerId:        serverID,
	})

	newData, err := proto.Marshal(&traceData)
	if err != nil {
		return fmt.Errorf("failed to marshal trace data: %w", err)
	}

	bm.Data = newData
	return nil
}
