package ocr

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"image"
	"image/jpeg"
	_ "image/png"
	"io"
	"net/http"
	"net/url"

	"golang.org/x/oauth2"
	"golang.org/x/oauth2/google"
)

type Recognizer interface {
	Recognize(context.Context, []byte) (string, error)
}

// PermanentError indicates that retrying the same image will not help.
type PermanentError struct{ Err error }

func (e *PermanentError) Error() string { return e.Err.Error() }
func (e *PermanentError) Unwrap() error { return e.Err }

func isPermanent(err error) bool {
	var permanent *PermanentError
	return errors.As(err, &permanent)
}

// Vision's inline JSON request limit is 10 MiB. Leave room for base64 and JSON.
const maxInlineImageBytes = 7 << 20

func prepareImage(data []byte) ([]byte, error) {
	if len(data) <= maxInlineImageBytes {
		return data, nil
	}
	decoded, _, err := image.Decode(bytes.NewReader(data))
	if err != nil {
		return nil, &PermanentError{fmt.Errorf("decode oversized screenshot: %w", err)}
	}
	var compressed bytes.Buffer
	if err := jpeg.Encode(&compressed, decoded, &jpeg.Options{Quality: 85}); err != nil {
		return nil, &PermanentError{fmt.Errorf("compress oversized screenshot: %w", err)}
	}
	if compressed.Len() > maxInlineImageBytes {
		return nil, &PermanentError{fmt.Errorf("screenshot remains too large for inline Vision OCR after compression")}
	}
	return compressed.Bytes(), nil
}

type VisionClient struct {
	client *http.Client
	url    string
}

type annotateRequest struct {
	Requests []annotateImage `json:"requests"`
}

type annotateImage struct {
	Image struct {
		Content []byte `json:"content"`
	} `json:"image"`
	Features []annotateFeature `json:"features"`
}

type annotateFeature struct {
	Type string `json:"type"`
}

func NewVisionClient(ctx context.Context, projectID string) (*VisionClient, error) {
	if projectID == "" {
		return nil, fmt.Errorf("GCP_PROJECT_ID is required when ENABLE_AI is set")
	}
	creds, err := google.FindDefaultCredentials(ctx, "https://www.googleapis.com/auth/cloud-vision")
	if err != nil {
		return nil, fmt.Errorf("find Vision credentials: %w", err)
	}
	return &VisionClient{
		client: oauth2.NewClient(ctx, creds.TokenSource),
		url:    "https://us-vision.googleapis.com/v1/projects/" + url.PathEscape(projectID) + "/locations/us/images:annotate",
	}, nil
}

func (c *VisionClient) Recognize(ctx context.Context, image []byte) (string, error) {
	image, err := prepareImage(image)
	if err != nil {
		return "", err
	}
	entry := annotateImage{Features: []annotateFeature{{Type: "DOCUMENT_TEXT_DETECTION"}}}
	entry.Image.Content = image
	body, err := json.Marshal(annotateRequest{Requests: []annotateImage{entry}})
	if err != nil {
		return "", fmt.Errorf("encode Vision request: %w", err)
	}
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, c.url, bytes.NewReader(body))
	if err != nil {
		return "", err
	}
	req.Header.Set("Content-Type", "application/json")
	res, err := c.client.Do(req)
	if err != nil {
		return "", fmt.Errorf("call Vision: %w", err)
	}
	defer res.Body.Close()
	data, err := io.ReadAll(io.LimitReader(res.Body, 16<<20))
	if err != nil {
		return "", fmt.Errorf("read Vision response: %w", err)
	}
	if res.StatusCode != http.StatusOK {
		err := fmt.Errorf("Vision returned HTTP %d: %s", res.StatusCode, data)
		if res.StatusCode == http.StatusBadRequest || res.StatusCode == http.StatusRequestEntityTooLarge {
			return "", &PermanentError{err}
		}
		return "", err
	}
	var result struct {
		Responses []struct {
			FullTextAnnotation struct {
				Text string `json:"text"`
			} `json:"fullTextAnnotation"`
			Error struct {
				Code    int    `json:"code"`
				Message string `json:"message"`
			} `json:"error"`
		} `json:"responses"`
	}
	if err := json.Unmarshal(data, &result); err != nil {
		return "", fmt.Errorf("decode Vision response: %w", err)
	}
	if len(result.Responses) != 1 {
		return "", fmt.Errorf("Vision returned %d responses", len(result.Responses))
	}
	if result.Responses[0].Error.Message != "" {
		err := fmt.Errorf("Vision OCR failed: %s", result.Responses[0].Error.Message)
		if result.Responses[0].Error.Code == 3 { // INVALID_ARGUMENT
			return "", &PermanentError{err}
		}
		return "", err
	}
	return result.Responses[0].FullTextAnnotation.Text, nil
}
