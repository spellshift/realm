package ocr

import (
	"bytes"
	"context"
	"crypto/rand"
	"encoding/json"
	"image"
	"image/png"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestVisionClientRecognize(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		require.Equal(t, http.MethodPost, r.Method)
		var request annotateRequest
		require.NoError(t, json.NewDecoder(r.Body).Decode(&request))
		require.Len(t, request.Requests, 1)
		require.Equal(t, []byte("image data"), request.Requests[0].Image.Content)
		require.Equal(t, "DOCUMENT_TEXT_DETECTION", request.Requests[0].Features[0].Type)
		_, _ = w.Write([]byte(`{"responses":[{"fullTextAnnotation":{"text":"hello world"}}]}`))
	}))
	defer server.Close()
	client := &VisionClient{client: server.Client(), url: server.URL}
	text, err := client.Recognize(context.Background(), []byte("image data"))
	require.NoError(t, err)
	require.Equal(t, "hello world", text)
}

func TestVisionClientCompressesOversizedPNG(t *testing.T) {
	pixels := image.NewRGBA(image.Rect(0, 0, 1600, 1600))
	_, err := rand.Read(pixels.Pix)
	require.NoError(t, err)
	var original bytes.Buffer
	require.NoError(t, png.Encode(&original, pixels))
	require.Greater(t, original.Len(), maxInlineImageBytes)
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		var request annotateRequest
		require.NoError(t, json.NewDecoder(r.Body).Decode(&request))
		content := request.Requests[0].Image.Content
		require.LessOrEqual(t, len(content), maxInlineImageBytes)
		require.True(t, bytes.HasPrefix(content, []byte{0xff, 0xd8}))
		_, _ = w.Write([]byte(`{"responses":[{"fullTextAnnotation":{"text":"large image"}}]}`))
	}))
	defer server.Close()
	client := &VisionClient{client: server.Client(), url: server.URL}
	text, err := client.Recognize(context.Background(), original.Bytes())
	require.NoError(t, err)
	require.Equal(t, "large image", text)
}
