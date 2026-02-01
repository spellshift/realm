package main

import (
	"context"
	"flag"
	"fmt"
	"log"
	"math"
	"os"
	"sort"
	"text/tabwriter"
	"time"

	"google.golang.org/protobuf/proto"

	"realm.pub/tavern/portals/portalpb"
	"realm.pub/tavern/portals/tracepb"
)

func traceCommand(args []string) {
	fs := flag.NewFlagSet("trace", flag.ExitOnError)
	portalID := fs.Int64("portal", 0, "Portal ID (required)")
	size := fs.Int("size", 0, "Payload padding size (bytes)")
	upstreamAddr := fs.String("upstream", "127.0.0.1:8000", "Upstream gRPC Address")
	count := fs.Int("count", 1, "Number of trace messages to send")

	if err := fs.Parse(args); err != nil {
		log.Fatal(err)
	}

	if *portalID == 0 {
		log.Fatal("--portal is required")
	}

	// Ensure at least one trace is run
	runCount := *count
	if runCount < 1 {
		runCount = 1
	}

	ctx := authGRPCContext(context.Background(), *upstreamAddr, authCachePath)

	runTrace(ctx, *upstreamAddr, *portalID, *size, runCount)
}

func runTrace(ctx context.Context, upstreamAddr string, portalID int64, size int, count int) {
	conn, err := connect(upstreamAddr)
	if err != nil {
		log.Fatalf("failed to run trace: %v", err)
	}
	defer conn.Close()

	client := portalpb.NewPortalClient(conn)

	stream, err := client.OpenPortal(ctx)
	if err != nil {
		log.Fatalf("Failed to open portal: %v", err)
	}

	if err := stream.Send(&portalpb.OpenPortalRequest{
		PortalId: portalID,
	}); err != nil {
		log.Fatalf("Failed to send registration message: %v", err)
	}

	var traces []*tracepb.TraceData

	for i := 0; i < count; i++ {
		// Basic progress for multiple runs
		if count > 1 {
			fmt.Fprintf(os.Stderr, "Sending trace %d/%d...\r", i+1, count)
		}

		td, err := traceOne(ctx, stream, portalID, size)
		if err != nil {
			log.Fatalf("\nTrace %d failed: %v", i+1, err)
		}
		traces = append(traces, td)
	}
	if count > 1 {
		fmt.Fprintln(os.Stderr, "")
	}

	if count <= 1 {
		printReport(traces[0])
	} else {
		stats, err := calculateStats(traces)
		if err != nil {
			log.Fatalf("Failed to calculate stats: %v", err)
		}
		printStats(stats, traces)
	}
}

func traceOne(ctx context.Context, stream portalpb.Portal_OpenPortalClient, portalID int64, size int) (*tracepb.TraceData, error) {
	start := time.Now().UTC().UnixMicro()

	if err := sendTraceMote(stream, portalID, size); err != nil {
		return nil, fmt.Errorf("failed to send trace mote: %w", err)
	}

	// Retry loop
	ctx, cancel := context.WithCancel(ctx)
	defer cancel()

	go func() {
		t := time.NewTicker(30 * time.Second)
		defer t.Stop()
		for {
			select {
			case <-ctx.Done():
				return
			case <-t.C:
				fmt.Fprintf(os.Stderr, "No reply yet, sending another trace...\n")
				if err := sendTraceMote(stream, portalID, size); err != nil {
					log.Printf("Failed to resend trace mote: %v", err)
				}
			}
		}
	}()

	for {
		resp, err := stream.Recv()
		if err != nil {
			return nil, fmt.Errorf("recv error: %w", err)
		}
		if resp.Mote == nil {
			continue
		}

		bm := resp.Mote.GetBytes()
		if bm != nil && bm.Kind == portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_TRACE {
			var traceData tracepb.TraceData
			if err := proto.Unmarshal(bm.Data, &traceData); err != nil {
				return nil, fmt.Errorf("failed to unmarshal trace data: %w", err)
			}

			// Ignore stale traces from previous iterations
			if traceData.StartMicros < start {
				continue
			}

			// Add USER_RECV event
			traceData.Events = append(traceData.Events, &tracepb.TraceEvent{
				Kind:            tracepb.TraceEventKind_TRACE_EVENT_KIND_USER_RECV,
				TimestampMicros: time.Now().UTC().UnixMicro(),
			})

			return &traceData, nil
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
		return fmt.Errorf("marshal trace data: %w", err)
	}

	mote := &portalpb.Mote{
		StreamId: "trace-stream",
		Payload: &portalpb.Mote_Bytes{
			Bytes: &portalpb.BytesPayload{
				Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_TRACE,
				Data: traceBytes,
			},
		},
	}

	// Add USER_SEND event
	mote, err = addTraceEvent(mote, tracepb.TraceEventKind_TRACE_EVENT_KIND_USER_SEND)
	if err != nil {
		return fmt.Errorf("add USER_SEND event: %w", err)
	}

	if err := stream.Send(&portalpb.OpenPortalRequest{
		PortalId: portalID,
		Mote:     mote,
	}); err != nil {
		return fmt.Errorf("send request: %w", err)
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

func printReport(traceData *tracepb.TraceData) {
	var totalDuration int64
	if len(traceData.Events) > 0 {
		totalDuration = traceData.Events[len(traceData.Events)-1].TimestampMicros - traceData.StartMicros
	} else {
		totalDuration = time.Now().UTC().UnixMicro() - traceData.StartMicros
	}

	fmt.Printf("\nTrace Report (Total Duration: %s)\n", formatDuration(totalDuration))

	w := tabwriter.NewWriter(os.Stdout, 0, 0, 2, ' ', 0)
	fmt.Fprintln(w, "Step Name\tTimestamp\tDelta")

	lastTime := traceData.StartMicros

	for _, evt := range traceData.Events {
		delta := evt.TimestampMicros - lastTime
		ts := time.UnixMicro(evt.TimestampMicros).UTC().Format("15:04:05.000000")
		fmt.Fprintf(w, "%s\t%s\t%s\n", evt.Kind.String(), ts, formatDuration(delta))
		lastTime = evt.TimestampMicros
	}
	w.Flush()
}

// Stats structures
type DistributionStats struct {
	Min, P50, P90, P99, Max int64
}

func calculateStats(traces []*tracepb.TraceData) (map[string]DistributionStats, error) {
	if len(traces) == 0 {
		return nil, fmt.Errorf("no traces to calculate stats from")
	}

	// Verify consistency
	firstLen := len(traces[0].Events)

	stepDeltas := make(map[string][]int64)
	totalDurations := make([]int64, 0, len(traces))

	for _, td := range traces {
		if len(td.Events) != firstLen {
			return nil, fmt.Errorf("trace event count mismatch")
		}

		lastTime := td.StartMicros
		for _, evt := range td.Events {
			delta := evt.TimestampMicros - lastTime
			kind := evt.Kind.String()
			stepDeltas[kind] = append(stepDeltas[kind], delta)
			lastTime = evt.TimestampMicros
		}

		totalDurations = append(totalDurations, lastTime-td.StartMicros)
	}

	results := make(map[string]DistributionStats)

	for kind, deltas := range stepDeltas {
		results[kind] = computeDist(deltas)
	}
	results["Total Duration"] = computeDist(totalDurations)

	return results, nil
}

func computeDist(values []int64) DistributionStats {
	sort.Slice(values, func(i, j int) bool { return values[i] < values[j] })
	n := len(values)
	return DistributionStats{
		Min: values[0],
		P50: values[int(math.Ceil(0.50*float64(n)))-1],
		P90: values[int(math.Ceil(0.90*float64(n)))-1],
		P99: values[int(math.Ceil(0.99*float64(n)))-1],
		Max: values[n-1],
	}
}

func printStats(stats map[string]DistributionStats, traces []*tracepb.TraceData) {
	fmt.Printf("\nTrace Statistics (Samples: %d)\n", len(traces))

	w := tabwriter.NewWriter(os.Stdout, 0, 0, 2, ' ', 0)
	// Headers
	fmt.Fprintln(w, "Step Name\tMin\tP50\tP90\tP99\tMax")

	// Use the first trace to determine order of steps
	for _, evt := range traces[0].Events {
		kind := evt.Kind.String()
		s := stats[kind]
		fmt.Fprintf(w, "%s\t%s\t%s\t%s\t%s\t%s\n",
			kind,
			formatDuration(s.Min),
			formatDuration(s.P50),
			formatDuration(s.P90),
			formatDuration(s.P99),
			formatDuration(s.Max))
	}

	// Total
	t := stats["Total Duration"]
	fmt.Fprintf(w, "Total Duration\t%s\t%s\t%s\t%s\t%s\n",
		formatDuration(t.Min),
		formatDuration(t.P50),
		formatDuration(t.P90),
		formatDuration(t.P99),
		formatDuration(t.Max))

	w.Flush()
}

func formatDuration(micros int64) string {
	if micros >= 1_000_000 {
		return fmt.Sprintf("%.4f seconds", float64(micros)/1_000_000.0)
	} else if micros >= 1_000 {
		return fmt.Sprintf("%.4fms", float64(micros)/1_000.0)
	}
	return fmt.Sprintf("%dµs", micros)
}
