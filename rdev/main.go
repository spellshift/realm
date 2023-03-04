package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"strconv"
	"text/tabwriter"
	"time"

	"github.com/kcarretto/realm/tavern/namegen"
	"github.com/urfave/cli"
)

const (
	configPath = ".rdev.json"
)

var (
	cmdArg        string
	sessionIDsArg cli.IntSlice
)

type Client struct {
	TavernAddr  string `json:"tavern-addr"`
	AuthSession string `json:"auth-session"`
	ExecTomeID  int    `json:"exec-tome-id"`
}

func newClient() (client Client) {
	cfgBytes, err := os.ReadFile(configPath)
	if err != nil {
		log.Fatalf("failed to read config: %v", err)
	}

	if err := json.Unmarshal(cfgBytes, &client); err != nil {
		log.Fatalf("failed to parse config: %v", err)
	}
	return
}

type session struct {
	ID         string    `json:"id"`
	Hostname   string    `json:"hostname"`
	LastSeenAt time.Time `json:"lastseenat"`
}

func (client *Client) getActiveSessions() (sessions []session) {
	query := graphQLQuery{
		OperationName: "Sessions",
		Query: `
			query Sessions {
				sessions {
					id
					hostname
					lastseenat
				}
			}`,
	}
	var resp struct {
		Data struct {
			Sessions []session `json:"sessions"`
		} `json:"data"`
	}
	client.do(query, &resp)
	for _, session := range resp.Data.Sessions {
		if time.Since(session.LastSeenAt) > 5*time.Minute {
			continue
		}
		sessions = append(sessions, session)
	}
	return
}

func (client *Client) getActiveSessionIDs() (ids []int) {
	sessions := client.getActiveSessions()
	for _, session := range sessions {
		id, err := strconv.Atoi(session.ID)
		if err != nil {
			log.Fatalf("failed to convert session id %q (host:%q) to integer: %v", session.ID, session.Hostname, err)
		}
		ids = append(ids, id)
	}
	return
}

func (client *Client) ShowActive() {
	// initialize tabwriter
	w := tabwriter.NewWriter(os.Stdout, 8, 8, 0, '\t', 0)
	defer w.Flush()

	logger := log.New(w, "", log.Flags())
	sessions := client.getActiveSessions()
	for _, session := range sessions {
		logger.Printf("%s\t(%s)\t%ds ago\t(%s)", session.Hostname, session.ID, time.Since(session.LastSeenAt)/time.Second, session.LastSeenAt.String())
	}
	logger.Printf("Found %d active sessions", len(sessions))
}

func (client *Client) Exec() {
	var targetSessions = sessionIDsArg.Value()
	if targetSessions == nil || len(targetSessions) < 1 {
		log.Printf("No --sessions provided, queuing for all active sessions, giving you 5 seconds to change your mind...")
		time.Sleep(5 * time.Second)
		log.Printf("Are you silly? I'm still gonna send it")
		targetSessions = client.getActiveSessionIDs()
	}

	query := graphQLQuery{
		OperationName: "CLIExecJob",
		Query: `
			mutation CLIExecJob($sessionIDs: [ID!]!, $input: CreateJobInput!) {
				createJob(sessionIDs: $sessionIDs, input: $input) {
					id
					name
				}
			}`,
		Variables: map[string]any{
			"sessionIDs": targetSessions,
			"input": map[string]any{
				"name":       namegen.GetRandomName(),
				"tomeID":     client.ExecTomeID,
				"parameters": fmt.Sprintf(`{"cmd":"%s"}`, cmdArg),
			},
		},
	}

	var resp struct {
		Data struct {
			Job struct {
				ID   string `json:"id"`
				Name string `json:"name"`
			} `json:"createJob"`
		} `json:"data"`
		Errors []struct {
			Message string `json:"message"`
		}
	}

	client.do(query, &resp)
	if resp.Errors != nil {
		for _, respErr := range resp.Errors {
			log.Printf("Failed: %s", respErr.Message)
		}
		log.Fatalf("failed with %d errors", len(resp.Errors))
	}
	log.Printf("Job (%s) queued on %d sessions: %s", resp.Data.Job.ID, (targetSessions), resp.Data.Job.Name)
}

type graphQLQuery struct {
	OperationName string         `json:"operationName"`
	Query         string         `json:"query"`
	Variables     map[string]any `json:"variables"`
}

func (client *Client) do(query graphQLQuery, dst any) {
	data, err := json.Marshal(query)
	if err != nil {
		log.Fatalf("failed to marshal graphql query to JSON: %v", err)
	}
	req, err := http.NewRequest(http.MethodPost, client.TavernAddr, bytes.NewBuffer(data))
	if err != nil {
		log.Fatalf("failed to create new http request: %v", err)
	}
	req.Header.Set("Content-Type", "application/json")
	req.AddCookie(&http.Cookie{
		Name:     "auth-session",
		Path:     "/",
		Value:    client.AuthSession,
		Expires:  time.Now().Add(7 * 24 * time.Hour),
		HttpOnly: true,
	})
	if client.AuthSession == "" {
		log.Fatalf("Please set a value for 'auth-session' in your .rdev.json configuration")
	}

	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		log.Fatalf("HTTP request failed: %v", err)
	}
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		log.Fatalf("Failed to read HTTP response: %v", err)
	}

	if err = json.Unmarshal(body, dst); err != nil {
		log.Fatalf("Failed to parse response: %v", err)
	}

}

func main() {
	client := newClient()

	app := cli.NewApp()
	app.Commands = []cli.Command{
		{
			Name: "show",
			Subcommands: []cli.Command{
				{
					Name: "active",
					Action: func(ctx *cli.Context) error {
						client.ShowActive()
						return nil
					},
				},
			},
		},
		{
			Name: "exec",
			Flags: []cli.Flag{
				cli.StringFlag{
					Name:        "cmd",
					Required:    true,
					Destination: &cmdArg,
				},
				cli.IntSliceFlag{
					Name:     "sessions",
					Required: false,
					Value:    &sessionIDsArg,
				},
			},
			Action: func(ctx *cli.Context) error {
				client.Exec()
				return nil
			},
		},
	}

	if err := app.Run(os.Args); err != nil {
		log.Fatalf("run failed: %v", err)
	}

}
