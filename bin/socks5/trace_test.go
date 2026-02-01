package main

import (
	"bytes"
	"io"
	"os"
	"strings"
	"testing"

	"realm.pub/tavern/portals/tracepb"
)

func TestFormatDuration(t *testing.T) {
	tests := []struct {
		micros int64
		want   string
	}{
		{1_543_200, "1.5432 seconds"},
		{123_543, "123.5430ms"},
		{40, "40µs"},
		{1_000, "1.0000ms"},
		{1_000_000, "1.0000 seconds"},
		{0, "0µs"},
	}

	for _, tt := range tests {
		got := formatDuration(tt.micros)
		if got != tt.want {
			t.Errorf("formatDuration(%d) = %q; want %q", tt.micros, got, tt.want)
		}
	}
}

func TestCalculateStats(t *testing.T) {
	// We simulate 3 traces.
	// Trace 1: Start=0. Ev1=10 (Delta=10). Ev2=30 (Delta=20). Total=30.
	// Trace 2: Start=0. Ev1=20 (Delta=20). Ev2=50 (Delta=30). Total=50.
	// Trace 3: Start=0. Ev1=30 (Delta=30). Ev2=40 (Delta=10). Total=40.

	// Deltas for Step 1: [10, 20, 30] -> Min=10, P50=20, P90=30, Max=30
	// Deltas for Step 2: [20, 30, 10] -> Sorted [10, 20, 30] -> Min=10, P50=20, P90=30, Max=30
	// Total Duration: [30, 50, 40] -> Sorted [30, 40, 50] -> Min=30, P50=40, Max=50

	t1 := &tracepb.TraceData{
		StartMicros: 1000,
		Events: []*tracepb.TraceEvent{
			{Kind: tracepb.TraceEventKind_TRACE_EVENT_KIND_USER_SEND, TimestampMicros: 1010},
			{Kind: tracepb.TraceEventKind_TRACE_EVENT_KIND_USER_RECV, TimestampMicros: 1030},
		},
	}
	t2 := &tracepb.TraceData{
		StartMicros: 2000,
		Events: []*tracepb.TraceEvent{
			{Kind: tracepb.TraceEventKind_TRACE_EVENT_KIND_USER_SEND, TimestampMicros: 2020},
			{Kind: tracepb.TraceEventKind_TRACE_EVENT_KIND_USER_RECV, TimestampMicros: 2050},
		},
	}
	t3 := &tracepb.TraceData{
		StartMicros: 3000,
		Events: []*tracepb.TraceEvent{
			{Kind: tracepb.TraceEventKind_TRACE_EVENT_KIND_USER_SEND, TimestampMicros: 3030},
			{Kind: tracepb.TraceEventKind_TRACE_EVENT_KIND_USER_RECV, TimestampMicros: 3040},
		},
	}

	traces := []*tracepb.TraceData{t1, t2, t3}

	stats, err := calculateStats(traces)
	if err != nil {
		t.Fatalf("calculateStats failed: %v", err)
	}

	// Check "Total Duration"
	total, ok := stats["Total Duration"]
	if !ok {
		t.Fatal("Missing Total Duration stats")
	}
	if total.Min != 30 {
		t.Errorf("Total Min: got %d, want 30", total.Min)
	}
	if total.P50 != 40 {
		t.Errorf("Total P50: got %d, want 40", total.P50)
	}
	if total.Max != 50 {
		t.Errorf("Total Max: got %d, want 50", total.Max)
	}

	// Check Step 1: USER_SEND (Delta from Start)
	// t1: 1010-1000 = 10
	// t2: 2020-2000 = 20
	// t3: 3030-3000 = 30
	s1, ok := stats["TRACE_EVENT_KIND_USER_SEND"]
	if !ok {
		t.Fatal("Missing USER_SEND stats")
	}
	if s1.Min != 10 {
		t.Errorf("S1 Min: got %d, want 10", s1.Min)
	}
	if s1.P50 != 20 {
		t.Errorf("S1 P50: got %d, want 20", s1.P50)
	}
	if s1.Max != 30 {
		t.Errorf("S1 Max: got %d, want 30", s1.Max)
	}

	// Check Step 2: USER_RECV (Delta from previous)
	// t1: 1030-1010 = 20
	// t2: 2050-2020 = 30
	// t3: 3040-3030 = 10
	// Sorted: 10, 20, 30
	s2, ok := stats["TRACE_EVENT_KIND_USER_RECV"]
	if !ok {
		t.Fatal("Missing USER_RECV stats")
	}
	if s2.Min != 10 {
		t.Errorf("S2 Min: got %d, want 10", s2.Min)
	}
	if s2.P50 != 20 {
		t.Errorf("S2 P50: got %d, want 20", s2.P50)
	}
	if s2.Max != 30 {
		t.Errorf("S2 Max: got %d, want 30", s2.Max)
	}
}

func TestPrintReport(t *testing.T) {
	// Capture stdout
	oldStdout := os.Stdout
	r, w, _ := os.Pipe()
	os.Stdout = w

	td := &tracepb.TraceData{
		StartMicros: 1000,
		Events: []*tracepb.TraceEvent{
			{
				Kind:            tracepb.TraceEventKind_TRACE_EVENT_KIND_USER_SEND,
				TimestampMicros: 1010,
				ServerId:        "test-client",
			},
			{
				Kind:            tracepb.TraceEventKind_TRACE_EVENT_KIND_SERVER_USER_RECV,
				TimestampMicros: 1020,
				ServerId:        "test-server-1",
			},
		},
	}

	printReport(td)

	w.Close()
	os.Stdout = oldStdout

	var buf bytes.Buffer
	io.Copy(&buf, r)
	output := buf.String()

	if !strings.Contains(output, "test-client") {
		t.Errorf("Expected output to contain 'test-client', got:\n%s", output)
	}
	if !strings.Contains(output, "test-server-1") {
		t.Errorf("Expected output to contain 'test-server-1', got:\n%s", output)
	}
}
