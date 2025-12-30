package main

import (
	"context"
	"flag"
	"fmt"
	"log"
	"os"
	"text/tabwriter"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/protobuf/proto"

	"realm.pub/tavern/portals/portalpb"
	"realm.pub/tavern/portals/tracepb"
)

func traceCommand(args []string) {
	fs := flag.NewFlagSet("trace", flag.ExitOnError)
	portalID := fs.Int64("portal", 0, "Portal ID (required)")
	size := fs.Int("size", 0, "Payload padding size (bytes)")
	upstreamAddr := fs.String("upstream_addr", "127.0.0.1:8000", "Upstream gRPC Address")

	if err := fs.Parse(args); err != nil {
		log.Fatal(err)
	}

	if *portalID == 0 {
		log.Fatal("--portal is required")
	}

	runTrace(*upstreamAddr, *portalID, *size)
}

func runTrace(upstreamAddr string, portalID int64, size int) {
	conn, err := grpc.NewClient(upstreamAddr, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		log.Fatalf("Failed to connect to upstream: %v", err)
	}
	defer conn.Close()

	client := portalpb.NewPortalClient(conn)
	ctx := context.Background()

	stream, err := client.OpenPortal(ctx)
	if err != nil {
		log.Fatalf("Failed to open portal: %v", err)
	}

	if err := sendTraceMote(stream, portalID, size); err != nil {
		log.Fatalf("Failed to send trace mote: %v", err)
	}
	fmt.Println("Trace sent, waiting for echo...")

	go func() {
		t := time.NewTicker(30 * time.Second)
		for {
			select {
			case <-ctx.Done():
				return
			case <-t.C:
				fmt.Printf("No reply yet, sending another trace...")
				if err := sendTraceMote(stream, portalID, size); err != nil {
					log.Fatalf("Failed to resend trace mote: %v", err)
				}
				continue
			}
		}
	}()

	for {
		resp, err := stream.Recv()
		if err != nil {
			log.Fatalf("Recv error: %v", err)
		}
		if resp.Mote == nil {
			continue
		}

		// Check if it's our trace mote
		if bm := resp.Mote.GetBytes(); bm != nil && bm.Kind == portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_TRACE {
			// 6. Add USER_RECV event (Proxy Read)
			// Note: We modify the mote locally before parsing for the report
			moteWithRecv, err := addTraceEvent(resp.Mote, tracepb.TraceEventKind_TRACE_EVENT_KIND_USER_RECV)
			if err != nil {
				log.Fatalf("Failed to add USER_RECV event: %v", err)
			}

			printReport(moteWithRecv)
			return
		}
	}
}

func sendTraceMote(stream portalpb.Portal_OpenPortalClient, portalID int64, size int) error {
	traceData := &tracepb.TraceData{
		StartMicros: time.Now().UTC().UnixMicro(),
		Padding:     make([]byte, size),
		Events:      []*tracepb.TraceEvent{},
	}

	traceBytes, err := proto.Marshal(traceData)
	if err != nil {
		log.Fatalf("Failed to marshal trace data: %v", err)
	}

	// 2. Wrap in Mote
	mote := &portalpb.Mote{
		StreamId: "trace-stream",
		Payload: &portalpb.Mote_Bytes{
			Bytes: &portalpb.BytesPayload{
				Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_TRACE,
				Data: traceBytes,
			},
		},
	}

	// 3. Add USER_SEND event (Proxy Write)
	mote, err = addTraceEvent(mote, tracepb.TraceEventKind_TRACE_EVENT_KIND_USER_SEND)
	if err != nil {
		log.Fatalf("Failed to add USER_SEND event: %v", err)
	}

	// 4. Send
	if err := stream.Send(&portalpb.OpenPortalRequest{
		PortalId: portalID,
		Mote:     mote,
	}); err != nil {
		log.Fatalf("Failed to send trace mote: %v", err)
	}

	return nil
}

func addTraceEvent(mote *portalpb.Mote, kind tracepb.TraceEventKind) (*portalpb.Mote, error) {
	bm := mote.GetBytes()
	if bm == nil || bm.Kind != portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_TRACE {
		return mote, nil
	}

	var traceData tracepb.TraceData
	if err := proto.Unmarshal(bm.Data, &traceData); err != nil {
		return nil, fmt.Errorf("failed to unmarshal trace data: %w", err)
	}

	traceData.Events = append(traceData.Events, &tracepb.TraceEvent{
		Kind:            kind,
		TimestampMicros: time.Now().UTC().UnixMicro(),
	})

	newData, err := proto.Marshal(&traceData)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal trace data: %w", err)
	}

	bm.Data = newData
	return mote, nil
}

func printReport(mote *portalpb.Mote) {
	bm := mote.GetBytes()
	var traceData tracepb.TraceData
	if err := proto.Unmarshal(bm.Data, &traceData); err != nil {
		log.Fatalf("Failed to unmarshal final trace data: %v", err)
	}

	fmt.Printf("\nTrace Report (Total Duration: %d µs)\n", time.Now().UTC().UnixMicro()-traceData.StartMicros)

	w := tabwriter.NewWriter(os.Stdout, 0, 0, 2, ' ', 0)
	fmt.Fprintln(w, "Step Name\tTimestamp\tDelta (µs)")

	lastTime := traceData.StartMicros

	for _, evt := range traceData.Events {
		delta := evt.TimestampMicros - lastTime
		ts := time.UnixMicro(evt.TimestampMicros).UTC().Format("15:04:05.000000")
		fmt.Fprintf(w, "%s\t%s\t%d\n", evt.Kind.String(), ts, delta)
		lastTime = evt.TimestampMicros
	}
	w.Flush()
}
