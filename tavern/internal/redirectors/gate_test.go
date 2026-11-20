package redirectors

import (
	"context"
	"testing"
)

func TestGateContext(t *testing.T) {
	if !Allowed(context.Background()) {
		t.Fatal("unmanaged traffic must remain enabled")
	}
	gate := NewGate()
	ctx := WithGate(context.Background(), gate)
	if !Allowed(ctx) {
		t.Fatal("managed traffic should start enabled")
	}
	gate.SetEnabled(false)
	if Allowed(ctx) {
		t.Fatal("disabled deployment accepted new work")
	}
	gate.SetEnabled(true)
	if !Allowed(ctx) {
		t.Fatal("re-enabled deployment rejected work")
	}
}
