package scheduler

import (
	"context"
	"net/url"
	"testing"
)

// testDriver is a simple Driver implementation for testing registration.
type testDriver struct{}

func (d *testDriver) Open(_ context.Context, _ *url.URL) (Scheduler, error) {
	return nil, nil
}

func TestRegister(t *testing.T) {
	// Reset global state for test isolation
	mu.Lock()
	origDrivers := drivers
	drivers = make(map[string]Driver)
	mu.Unlock()
	defer func() {
		mu.Lock()
		drivers = origDrivers
		mu.Unlock()
	}()

	Register("test", &testDriver{})

	list := Drivers()
	if len(list) != 1 || list[0] != "test" {
		t.Fatalf("expected [test], got %v", list)
	}
}

func TestRegisterNilPanics(t *testing.T) {
	defer func() {
		if r := recover(); r == nil {
			t.Fatal("expected panic for nil driver")
		}
	}()
	Register("nil-driver", nil)
}

func TestRegisterDuplicatePanics(t *testing.T) {
	mu.Lock()
	origDrivers := drivers
	drivers = make(map[string]Driver)
	mu.Unlock()
	defer func() {
		mu.Lock()
		drivers = origDrivers
		mu.Unlock()
	}()

	Register("dup", &testDriver{})

	defer func() {
		if r := recover(); r == nil {
			t.Fatal("expected panic for duplicate registration")
		}
	}()
	Register("dup", &testDriver{})
}

func TestNewUnknownDriver(t *testing.T) {
	_, err := New(context.Background(), "unknown://foo")
	if err == nil {
		t.Fatal("expected error for unknown driver")
	}
}

func TestNewMissingScheme(t *testing.T) {
	_, err := New(context.Background(), "no-scheme")
	if err == nil {
		t.Fatal("expected error for missing scheme")
	}
}
