package portals_test

import (
	"context"
	"sync"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/proto"
	"realm.pub/tavern/internal/portals"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/portals/portalpb"
	"realm.pub/tavern/portals/tracepb"
)

func newTraceMote(t *testing.T) *portalpb.Mote {
	t.Helper()
	raw, err := proto.Marshal(&tracepb.TraceData{
		StartMicros: time.Now().UTC().UnixMicro(),
	})
	require.NoError(t, err)
	return &portalpb.Mote{
		StreamId: "trace-stream",
		Payload: &portalpb.Mote_Bytes{
			Bytes: &portalpb.BytesPayload{
				Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_TRACE,
				Data: raw,
			},
		},
	}
}

func TestAddTraceEvent_DoesNotMutateOriginal(t *testing.T) {
	orig := newTraceMote(t)
	origBytes := append([]byte(nil), orig.GetBytes().Data...)

	updated, err := portals.AddTraceEvent(orig, tracepb.TraceEventKind_TRACE_EVENT_KIND_SERVER_USER_RECV)
	require.NoError(t, err)
	require.NotSame(t, orig, updated)

	assert.Equal(t, origBytes, orig.GetBytes().Data, "original mote data must remain unchanged")

	var td tracepb.TraceData
	require.NoError(t, proto.Unmarshal(updated.GetBytes().Data, &td))
	require.Len(t, td.Events, 1)
	assert.Equal(t, tracepb.TraceEventKind_TRACE_EVENT_KIND_SERVER_USER_RECV, td.Events[0].Kind)
}

func TestAddTraceEvent_ConcurrentPublishAndSubscribeNoRace(t *testing.T) {
	m := mux.New()
	ctx := context.Background()
	topic := "trace-race-topic"

	const subscribers = 4
	const messages = 50

	var wg sync.WaitGroup
	cancels := make([]func(), subscribers)
	for i := 0; i < subscribers; i++ {
		ch, cancel := m.Subscribe(topic)
		cancels[i] = cancel
		wg.Add(1)
		go func(c <-chan *portalpb.Mote) {
			defer wg.Done()
			for j := 0; j < messages; j++ {
				mote := <-c
				traced, err := portals.AddTraceEvent(mote, tracepb.TraceEventKind_TRACE_EVENT_KIND_SERVER_USER_SUB)
				assert.NoError(t, err)
				traced, err = portals.AddTraceEvent(traced, tracepb.TraceEventKind_TRACE_EVENT_KIND_SERVER_USER_SEND)
				assert.NoError(t, err)
				_, err = proto.Marshal(traced)
				assert.NoError(t, err)
			}
		}(ch)
	}
	defer func() {
		for _, c := range cancels {
			c()
		}
	}()

	for j := 0; j < messages; j++ {
		mote := newTraceMote(t)
		traced, err := portals.AddTraceEvent(mote, tracepb.TraceEventKind_TRACE_EVENT_KIND_SERVER_USER_PUB)
		require.NoError(t, err)
		require.NoError(t, m.Publish(ctx, topic, traced))
	}

	wg.Wait()
}
