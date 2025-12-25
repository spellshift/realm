package http1

import (
	"bytes"
	"context"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

func TestRequirePOST(t *testing.T) {
	// Case 1: Method is POST
	req := httptest.NewRequest(http.MethodPost, "/", nil)
	w := httptest.NewRecorder()

	valid := requirePOST(w, req)
	assert.True(t, valid)
	assert.Equal(t, http.StatusOK, w.Code) // No error written

	// Case 2: Method is GET
	req = httptest.NewRequest(http.MethodGet, "/", nil)
	w = httptest.NewRecorder()

	valid = requirePOST(w, req)
	assert.False(t, valid)
	assert.Equal(t, http.StatusMethodNotAllowed, w.Code)
}

func TestReadRequestBody(t *testing.T) {
	// Case 1: Valid body
	bodyContent := []byte("hello world")
	req := httptest.NewRequest(http.MethodPost, "/", bytes.NewReader(bodyContent))
	w := httptest.NewRecorder()

	data, ok := readRequestBody(w, req)
	assert.True(t, ok)
	assert.Equal(t, bodyContent, data)

	// Case 2: Read error (simulated by closing the body prematurely or custom reader)
	// Using a custom reader that fails on Read
	failReader := &ErrorReader{}
	req = httptest.NewRequest(http.MethodPost, "/", failReader)
	w = httptest.NewRecorder()

	data, ok = readRequestBody(w, req)
	assert.False(t, ok)
	assert.Nil(t, data)
	assert.Equal(t, http.StatusBadRequest, w.Code)
}

type ErrorReader struct{}
func (e *ErrorReader) Read(p []byte) (n int, err error) {
	return 0, assert.AnError
}

func TestSetGRPCResponseHeaders(t *testing.T) {
	w := httptest.NewRecorder()
	setGRPCResponseHeaders(w)

	assert.Equal(t, "application/grpc", w.Header().Get("Content-Type"))
	assert.Equal(t, http.StatusOK, w.Code)
}

func TestGetFlusher(t *testing.T) {
	// Case 1: ResponseWriter supports Flusher (httptest.ResponseRecorder does NOT implement Flusher by default in all Go versions,
	// but check if we can mock it or if NewRecorder supports it now.
	// Actually NewRecorder implements Flusher starting Go 1.6+)
	w := httptest.NewRecorder()
	f, ok := getFlusher(w)
	assert.True(t, ok)
	assert.NotNil(t, f)

	// Case 2: ResponseWriter does not support Flusher
	mw := &MockWriter{}
	f, ok = getFlusher(mw)
	assert.False(t, ok)
	assert.Nil(t, f)
}

type MockWriter struct{}
func (m *MockWriter) Header() http.Header { return http.Header{} }
func (m *MockWriter) Write([]byte) (int, error) { return 0, nil }
func (m *MockWriter) WriteHeader(statusCode int) {}

func TestHandleStreamError(t *testing.T) {
	w := httptest.NewRecorder()
	handleStreamError(w, "error msg", assert.AnError)

	assert.Equal(t, http.StatusInternalServerError, w.Code)
	assert.Contains(t, w.Body.String(), "error msg")
}

func TestCreateRequestContext(t *testing.T) {
	timeout := 100 * time.Millisecond
	ctx, cancel := createRequestContext(timeout)
	defer cancel()

	assert.NotNil(t, ctx)

	// Verify timeout
	select {
	case <-ctx.Done():
		// Should not be done immediately
		assert.Fail(t, "context closed too early")
	default:
		// OK
	}

	// Wait for timeout
	time.Sleep(150 * time.Millisecond)
	assert.ErrorIs(t, ctx.Err(), context.DeadlineExceeded)
}
