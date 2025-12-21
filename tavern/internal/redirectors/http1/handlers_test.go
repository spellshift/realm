package http1

import (
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestGetClientIP(t *testing.T) {
	tests := []struct {
		name           string
		setupRequest   func() *http.Request
		expectedIP     string
	}{
		{
			name: "X-Forwarded-For_Set",
			setupRequest: func() *http.Request {
				req := httptest.NewRequest(http.MethodPost, "/test", nil)
				req.Header.Set("X-Forwarded-For", "203.0.113.42")
				req.RemoteAddr = "192.0.2.1:12345"
				return req
			},
			expectedIP: "203.0.113.42",
		},
		{
			name: "X-Forwarded-For_Not_Set_IPv4",
			setupRequest: func() *http.Request {
				req := httptest.NewRequest(http.MethodPost, "/test", nil)
				req.RemoteAddr = "1.1.1.1:12345"
				return req
			},
			expectedIP: "1.1.1.1",
		},
		{
			name: "X-Forwarded-For_Not_Set_IPv6",
			setupRequest: func() *http.Request {
				req := httptest.NewRequest(http.MethodPost, "/test", nil)
				req.RemoteAddr = "[2001:db8::1]:12345"
				return req
			},
			expectedIP: "2001:db8::1",
		},
		{
			name: "X-Forwarded-For_Not_Set_IPv6_Localhost",
			setupRequest: func() *http.Request {
				req := httptest.NewRequest(http.MethodPost, "/test", nil)
				req.RemoteAddr = "[::1]:5000"
				return req
			},
			expectedIP: "::1",
		},
		{
			name: "X-Forwarded-For_Empty_Falls_Back_To_RemoteAddr",
			setupRequest: func() *http.Request {
				req := httptest.NewRequest(http.MethodPost, "/test", nil)
				req.Header.Set("X-Forwarded-For", "")
				req.RemoteAddr = "1.1.1.1:12345"
				return req
			},
			expectedIP: "1.1.1.1",
		},
		{
			name: "X-Forwarded-For_With_Multiple_IPs",
			setupRequest: func() *http.Request {
				req := httptest.NewRequest(http.MethodPost, "/test", nil)
				req.Header.Set("X-Forwarded-For", "203.0.113.42, 198.51.100.1, 192.0.2.5")
				req.RemoteAddr = "192.0.2.1:12345"
				return req
			},
			expectedIP: "203.0.113.42",
		},
		{
			name: "X-Forwarded-For_Malformed_IP",
			setupRequest: func() *http.Request {
				req := httptest.NewRequest(http.MethodPost, "/test", nil)
				req.Header.Set("X-Forwarded-For", "not-an-ip")
				req.RemoteAddr = "1.1.1.1"
				return req
			},
			expectedIP: "unknown",
		},
		{
			name: "RemoteAddr_Without_Port",
			setupRequest: func() *http.Request {
				req := httptest.NewRequest(http.MethodPost, "/test", nil)
				req.RemoteAddr = "1.1.1.1"
				return req
			},
			expectedIP: "unknown",
		},
		{
			name: "RemoteAddr_Multiple_Colons_IPv6_With_Port",
			setupRequest: func() *http.Request {
				req := httptest.NewRequest(http.MethodPost, "/test", nil)
				req.RemoteAddr = "2001:db8::1:5000"
				return req
			},
			expectedIP: "2001:db8::1",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			req := tt.setupRequest()
			result := getClientIP(req)
			if result != tt.expectedIP {
				t.Errorf("getClientIP() = %v, want %v", result, tt.expectedIP)
			}
		})
	}
}
