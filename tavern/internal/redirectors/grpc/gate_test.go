package grpc

import (
	"context"
	"testing"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/redirectors"
)

func TestDisabledStreamRejectedBeforeForwarding(t *testing.T) {
	gate := redirectors.NewGate()
	gate.SetEnabled(false)
	ctx := redirectors.WithGate(context.Background(), gate)
	err := (&Redirector{}).handler(nil, ctx)(nil, nil)
	if status.Code(err) != codes.Unavailable {
		t.Fatalf("disabled stream returned %v", err)
	}
}
