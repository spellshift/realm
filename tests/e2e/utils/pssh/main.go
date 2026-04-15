package main

import (
	"bufio"
	"encoding/json"
	"flag"
	"fmt"
	"log"
	"net/url"
	"os"

	"github.com/gorilla/websocket"
	"golang.org/x/term"
)

type WebsocketMessageKind string

const (
	WebsocketMessageKindInput     = "INPUT"
	WebsocketMessageKindOutput    = "OUTPUT"
	WebsocketMessageKindTaskError = "TASK_ERROR"
	WebsocketMessageKindError     = "ERROR"
)

type WebsocketTaskInputMessage struct {
	Kind  WebsocketMessageKind `json:"kind"`
	Input string               `json:"input"`
}

type WebsocketTaskOutputMessage struct {
	Kind   WebsocketMessageKind `json:"kind"`
	Output string               `json:"output"`
}

type WebsocketErrorMessage struct {
	Kind  WebsocketMessageKind `json:"kind"`
	Error string               `json:"error"`
}

func main() {
	portalID := flag.String("portal", "", "Portal ID")
	target := flag.String("target", "", "Target SSH Server (user:pass@host:port)")
	wsURL := flag.String("url", "ws://localhost/portals/ssh/ws", "WebSocket Endpoint URL")
	flag.Parse()

	if *portalID == "" || *target == "" {
		log.Fatalf("Usage: pssh -portal <portal_id> -target <target> [-url <websocket_url>]")
	}

	u, err := url.Parse(*wsURL)
	if err != nil {
		log.Fatalf("Invalid websocket URL: %v", err)
	}
	q := u.Query()
	q.Set("portal_id", *portalID)
	q.Set("target", *target)
	u.RawQuery = q.Encode()

	// Connect
	c, resp, err := websocket.DefaultDialer.Dial(u.String(), nil)
	if err != nil {
		if resp != nil {
			log.Fatalf("Failed to dial websocket: %v (Status: %s)", err, resp.Status)
		} else {
			log.Fatalf("Failed to dial websocket: %v", err)
		}
	}
	defer c.Close()

	// Setup Terminal in Raw Mode
	fd := int(os.Stdin.Fd())
	oldState, err := term.MakeRaw(fd)
	if err != nil {
		log.Fatalf("Failed to set raw mode: %v", err)
	}
	defer term.Restore(fd, oldState)

	// Receiver loop
	go func() {
		for {
			_, message, err := c.ReadMessage()
			if err != nil {
				// We don't log cleanly because we are in raw mode, but a print is fine
				fmt.Printf("\r\n[!] Disconnected: %v\r\n", err)
				term.Restore(fd, oldState)
				os.Exit(0)
				return
			}

			// We need to unmarshal to check what it is
			// Because we can have OUTPUT or TASK_ERROR or ERROR
			var generic map[string]interface{}
			if err := json.Unmarshal(message, &generic); err == nil {
				kind, _ := generic["kind"].(string)
				switch kind {
				case WebsocketMessageKindOutput, WebsocketMessageKindTaskError:
					if out, ok := generic["output"].(string); ok {
						fmt.Print(out)
					}
					if out, ok := generic["error"].(string); ok {
						fmt.Print(out)
					}
				case WebsocketMessageKindError:
					if out, ok := generic["error"].(string); ok {
						fmt.Printf("\r\n[ERROR] %s\r\n", out)
					}
				}
			}
		}
	}()

	// Read from stdin byte-by-byte
	reader := bufio.NewReader(os.Stdin)
	for {
		b, err := reader.ReadByte()
		if err != nil {
			fmt.Printf("\r\nError reading stdin: %v\r\n", err)
			break
		}

		msg := WebsocketTaskInputMessage{
			Kind:  WebsocketMessageKindInput,
			Input: string(b),
		}
		data, err := json.Marshal(msg)
		if err != nil {
			continue
		}

		if err := c.WriteMessage(websocket.TextMessage, data); err != nil {
			fmt.Printf("\r\nError writing message: %v\r\n", err)
			break
		}
	}
}
