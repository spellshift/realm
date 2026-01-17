package http1

import (
	"bytes"
	"io"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

func TestRequirePOST(t *testing.T) {
	tests := []struct {
		method string
		want   bool
		status int
	}{
		{http.MethodPost, true, http.StatusOK},
		{http.MethodGet, false, http.StatusMethodNotAllowed},
		{http.MethodPut, false, http.StatusMethodNotAllowed},
	}

	for _, tt := range tests {
		t.Run(tt.method, func(t *testing.T) {
			w := httptest.NewRecorder()
			r := httptest.NewRequest(tt.method, "/", nil)

			got := requirePOST(w, r)
			assert.Equal(t, tt.want, got)
			if !got {
				assert.Equal(t, tt.status, w.Code)
			}
		})
	}
}

func TestReadRequestBody(t *testing.T) {
	t.Run("Valid body", func(t *testing.T) {
		body := []byte("test body")
		w := httptest.NewRecorder()
		r := httptest.NewRequest(http.MethodPost, "/", bytes.NewReader(body))

		got, ok := readRequestBody(w, r)
		assert.True(t, ok)
		assert.Equal(t, body, got)
	})

	t.Run("Error reading body", func(t *testing.T) {
		// Mock a reader that returns an error
		w := httptest.NewRecorder()
		r := httptest.NewRequest(http.MethodPost, "/", &errorReader{})

		got, ok := readRequestBody(w, r)
		assert.False(t, ok)
		assert.Nil(t, got)
		assert.Equal(t, http.StatusBadRequest, w.Code)
	})
}

type errorReader struct{}

func (e *errorReader) Read(p []byte) (n int, err error) {
	return 0, io.ErrUnexpectedEOF
}

func TestSetGRPCResponseHeaders(t *testing.T) {
	w := httptest.NewRecorder()
	setGRPCResponseHeaders(w)

	assert.Equal(t, "application/grpc", w.Header().Get("Content-Type"))
	assert.Equal(t, http.StatusOK, w.Code)
}

func TestGetFlusher(t *testing.T) {
	t.Run("Supports Flusher", func(t *testing.T) {
		w := httptest.NewRecorder()
		_, ok := getFlusher(w)
		assert.True(t, ok)
	})

	// It's hard to mock a ResponseWriter that *doesn't* support Flusher using httptest.NewRecorder
	// because it implements it. We'd need a custom struct.
	t.Run("Does Not Support Flusher", func(t *testing.T) {
		w := &basicResponseWriter{}
		_, ok := getFlusher(w)
		assert.False(t, ok)
	})
}

type basicResponseWriter struct{}

func (b *basicResponseWriter) Header() http.Header       { return http.Header{} }
func (b *basicResponseWriter) Write([]byte) (int, error) { return 0, nil }
func (b *basicResponseWriter) WriteHeader(statusCode int) {}

func TestHandleStreamError(t *testing.T) {
	w := httptest.NewRecorder()
	handleStreamError(w, "stream failed", io.ErrUnexpectedEOF)

	assert.Equal(t, http.StatusInternalServerError, w.Code)
	assert.Contains(t, w.Body.String(), "stream failed")
	assert.Contains(t, w.Body.String(), "unexpected EOF")
}

func TestCreateRequestContext(t *testing.T) {
	timeout := 100 * time.Millisecond
	ctx, cancel := createRequestContext(timeout)
	defer cancel()

	assert.NotNil(t, ctx)
	assert.NotNil(t, cancel)

	deadline, ok := ctx.Deadline()
	assert.True(t, ok)
	assert.WithinDuration(t, time.Now().Add(timeout), deadline, 10*time.Millisecond)
}
