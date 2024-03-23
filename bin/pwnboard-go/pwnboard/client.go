package pwnboard

import (
	"bytes"
	"fmt"
	"io"

	"encoding/json"
	"net/http"
)

var (
	BOX_ACCESS_PATH = "/pwn/boxaccess"
)

type Request struct {
	Data map[string]any
}

type Client struct {
	ApplicationName string
	URL             string
	HTTP            *http.Client
}

func (c *Client) ReportIPs(ips []string) error {
	if len(ips) == 0 {
		return nil
	}
	var r Request
	if len(ips) == 1 {
		r = Request{
			Data: map[string]any{
				"ip":          ips[0],
				"application": c.ApplicationName,
			},
		}
	} else {
		r = Request{
			Data: map[string]any{
				"ip":          ips[0],
				"application": c.ApplicationName,
				"ips":         ips[1:],
			},
		}
	}
	return c.do(r)
}

func (c *Client) do(r Request) error {
	data, err := json.Marshal(r.Data)
	if err != nil {
		return fmt.Errorf("failed to marshal json request to json: %w", err)
	}

	req, err := http.NewRequest(http.MethodPost, c.URL+BOX_ACCESS_PATH, bytes.NewBuffer(data))
	if err != nil {
		return err
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := c.HTTP.Do(req)
	if err != nil {
		return fmt.Errorf("failed to send request: %v", err)
	}

	// if pwnboard issue
	if resp.StatusCode != 202 {
		body, err := io.ReadAll(resp.Body)
		if err != nil {
			return err
		}
		return fmt.Errorf("pwnboard error: %s", body)
	}
	return nil
}
