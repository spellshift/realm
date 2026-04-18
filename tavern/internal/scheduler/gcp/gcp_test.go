package gcp

import (
	"testing"

	schedulerpb "cloud.google.com/go/scheduler/apiv1/schedulerpb"
	"realm.pub/tavern/internal/scheduler"
)

func TestDriverRegistered(t *testing.T) {
	for _, name := range scheduler.Drivers() {
		if name == "gcp" {
			return
		}
	}
	t.Fatal("gcp driver not registered")
}

func TestHTTPMethodToProto(t *testing.T) {
	tests := []struct {
		method string
		want   schedulerpb.HttpMethod
	}{
		{"GET", schedulerpb.HttpMethod_GET},
		{"POST", schedulerpb.HttpMethod_POST},
		{"PUT", schedulerpb.HttpMethod_PUT},
		{"DELETE", schedulerpb.HttpMethod_DELETE},
		{"PATCH", schedulerpb.HttpMethod_PATCH},
		{"HEAD", schedulerpb.HttpMethod_HEAD},
		{"OPTIONS", schedulerpb.HttpMethod_OPTIONS},
		{"get", schedulerpb.HttpMethod_GET},
		{"", schedulerpb.HttpMethod_HTTP_METHOD_UNSPECIFIED},
		{"INVALID", schedulerpb.HttpMethod_HTTP_METHOD_UNSPECIFIED},
	}
	for _, tt := range tests {
		t.Run(tt.method, func(t *testing.T) {
			got := httpMethodToProto(tt.method)
			if got != tt.want {
				t.Errorf("httpMethodToProto(%q) = %v, want %v", tt.method, got, tt.want)
			}
		})
	}
}
