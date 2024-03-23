package tavern

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"

	"realm.pub/tavern/cli/auth"
)

// Request represents an outgoing GraphQL request
type Request struct {
	Query         string         `json:"query"`
	Variables     map[string]any `json:"variables,omitempty"`
	OperationName string         `json:"operationName,omitempty"`
	Extensions    map[string]any `json:"extensions,omitempty"`
}

// Response is a GraphQL layer response from a handler.
type Response struct {
	Errors []struct {
		Message string `json:"message"`
	} `json:"errors"`
	Extensions map[string]any
}

func (r Response) Error() string {
	msg := ""
	for _, err := range r.Errors {
		msg = fmt.Sprintf("%s\n%s;", msg, err.Message)
	}
	return msg
}

type Host struct {
	ID        string `json:"id"`
	Name      string `json:"name"`
	PrimaryIP string `json:"primaryIP"`
}

type Client struct {
	Credential auth.Token
	URL        string
	HTTP       *http.Client
}

func (c *Client) GetHostsSeenInLastDuration(timeAgo time.Duration) ([]Host, error) {
	now := time.Now().UTC()
	timeAgoFromNow := now.Add(-timeAgo)
	formattedTime := timeAgoFromNow.Format(time.RFC3339)
	req := Request{
		OperationName: "getHosts",
		Query: `query getHosts($input: HostWhereInput) {
			hosts(where: $input) {
				id
				primaryIP
				name
			}
		}`,
		Variables: map[string]any{
			"input": map[string]string{"lastSeenAtGT": formattedTime},
		},
	}

	type GetHostsResponse struct {
		Response
		Data struct {
			Hosts []Host `json:"hosts"`
		} `json:"data"`
	}
	var resp GetHostsResponse
	if err := c.do(req, &resp); err != nil {
		return nil, fmt.Errorf("http request failed: %w", err)
	}

	if resp.Errors != nil {
		return nil, fmt.Errorf("graphql error: %s", resp.Error())
	}

	return resp.Data.Hosts, nil
}

// do sends a GraphQL request and returns the response
func (c *Client) do(gqlReq Request, gqlResp any) error {

	data, err := json.Marshal(gqlReq)
	if err != nil {
		return fmt.Errorf("failed to marshal json request to json: %w", err)
	}

	req, err := http.NewRequest(http.MethodPost, c.URL, bytes.NewBuffer(data))
	if err != nil {
		return err
	}
	req.Header.Set("Content-Type", "application/json")
	c.Credential.Authenticate(req)

	resp, err := c.HTTP.Do(req)
	if err != nil {
		return fmt.Errorf("failed to send request: %v", err)
	}
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return err
	}

	if err := json.Unmarshal(body, gqlResp); err != nil {
		return fmt.Errorf("failed to unmarshal body to json: %v", err)
	}

	return nil
}
