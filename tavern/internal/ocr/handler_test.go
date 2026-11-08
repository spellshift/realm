package ocr_test

import (
	"context"
	"errors"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/c2/c2test"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/ocr"
)

type fakeRecognizer struct {
	text  string
	err   error
	calls int
}

type waitingRecognizer struct {
	started chan struct{}
	release chan struct{}
}

func (w *waitingRecognizer) Recognize(_ context.Context, _ []byte) (string, error) {
	close(w.started)
	<-w.release
	return "stale text", nil
}

func (f *fakeRecognizer) Recognize(_ context.Context, _ []byte) (string, error) {
	f.calls++
	return f.text, f.err
}

func TestWorkerWritesOCRTextWithoutChangingImage(t *testing.T) {
	ctx := context.Background()
	graph := enttest.OpenTempDB(t)
	beacon := c2test.NewRandomBeacon(ctx, graph)
	host := beacon.QueryHost().OnlyX(ctx)
	image := graph.Screenshot.Create().
		SetHost(host).
		SetName("screen.png").
		SetContent([]byte("image data")).
		SetOcrRequested(true).
		SaveX(ctx)
	fake := &fakeRecognizer{text: "recognized text"}
	handler := ocr.NewHandler(graph, fake, ocr.VerifyLoopback)
	unauthorized := httptest.NewRecorder()
	handler.ServeHTTP(unauthorized, httptest.NewRequest(http.MethodPost, ocr.Audience, nil))
	require.Equal(t, http.StatusUnauthorized, unauthorized.Code)
	require.Equal(t, 0, fake.calls)

	req := httptest.NewRequest(http.MethodPost, ocr.Audience, nil)
	req.RemoteAddr = "127.0.0.1:12345"
	res := httptest.NewRecorder()
	handler.ServeHTTP(res, req)
	require.Equal(t, http.StatusOK, res.Code)
	updated := graph.Screenshot.GetX(ctx, image.ID)
	require.Equal(t, "recognized text", *updated.OcrText)
	require.False(t, updated.OcrRequested)
	require.Equal(t, uint64(len("image data")), updated.Size)
	require.Equal(t, image.Hash, updated.Hash)

	res = httptest.NewRecorder()
	handler.ServeHTTP(res, req)
	require.Equal(t, http.StatusOK, res.Code)
	require.Equal(t, 1, fake.calls)
}

func TestWorkerRetriesFailedOCR(t *testing.T) {
	ctx := context.Background()
	graph := enttest.OpenTempDB(t)
	beacon := c2test.NewRandomBeacon(ctx, graph)
	host := beacon.QueryHost().OnlyX(ctx)
	image := graph.Screenshot.Create().SetHost(host).SetName("screen.png").SetContent([]byte("image data")).SetOcrRequested(true).SaveX(ctx)
	fake := &fakeRecognizer{err: errors.New("temporary error")}
	handler := ocr.NewHandler(graph, fake, ocr.VerifyLoopback)
	req := httptest.NewRequest(http.MethodPost, ocr.Audience, nil)
	req.RemoteAddr = "127.0.0.1:12345"

	for range 4 {
		res := httptest.NewRecorder()
		handler.ServeHTTP(res, req)
		require.Equal(t, http.StatusOK, res.Code)
		updated := graph.Screenshot.GetX(ctx, image.ID)
		require.True(t, updated.OcrLeaseUntil.After(time.Now()))
		graph.Screenshot.UpdateOneID(image.ID).SetOcrLeaseUntil(time.Now().Add(-time.Second)).ExecX(ctx)
	}
	require.Equal(t, 4, fake.calls)
	updated := graph.Screenshot.GetX(ctx, image.ID)
	require.True(t, updated.OcrRequested)
	require.Equal(t, 4, updated.OcrAttempts)
	require.Nil(t, updated.OcrText)
	fake.err = nil
	fake.text = "eventually recognized"
	res := httptest.NewRecorder()
	handler.ServeHTTP(res, req)
	require.Equal(t, http.StatusOK, res.Code)
	require.Equal(t, 5, fake.calls)
	updated = graph.Screenshot.GetX(ctx, image.ID)
	require.Equal(t, "eventually recognized", *updated.OcrText)
	require.False(t, updated.OcrRequested)
}

func TestStaleWorkerCannotOverwriteNewClaim(t *testing.T) {
	ctx := context.Background()
	graph := enttest.OpenTempDB(t)
	beacon := c2test.NewRandomBeacon(ctx, graph)
	host := beacon.QueryHost().OnlyX(ctx)
	image := graph.Screenshot.Create().SetHost(host).SetName("screen.png").SetContent([]byte("image data")).SetOcrRequested(true).SaveX(ctx)
	recognizer := &waitingRecognizer{started: make(chan struct{}), release: make(chan struct{})}
	handler := ocr.NewHandler(graph, recognizer, ocr.VerifyLoopback)
	req := httptest.NewRequest(http.MethodPost, ocr.Audience, nil)
	req.RemoteAddr = "127.0.0.1:12345"
	done := make(chan struct{})
	go func() {
		defer close(done)
		handler.ServeHTTP(httptest.NewRecorder(), req)
	}()
	<-recognizer.started
	newLease := time.Now().Add(2 * time.Minute)
	graph.Screenshot.UpdateOneID(image.ID).AddOcrAttempts(1).SetOcrLeaseUntil(newLease).ExecX(ctx)
	close(recognizer.release)
	<-done
	updated := graph.Screenshot.GetX(ctx, image.ID)
	require.True(t, updated.OcrRequested)
	require.Equal(t, 2, updated.OcrAttempts)
	require.Nil(t, updated.OcrText)
	require.WithinDuration(t, newLease, *updated.OcrLeaseUntil, time.Second)
}
