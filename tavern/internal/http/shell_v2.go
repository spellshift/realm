package http

import (
	"encoding/json"
	"fmt"
	"net/http"

	"github.com/gorilla/websocket"
)

var shellV2Upgrader = websocket.Upgrader{
	CheckOrigin: func(r *http.Request) bool {
		return true // Allow all origins for dev/testing
	},
}

type ShellV2Handler struct{}

func NewShellV2Handler() *ShellV2Handler {
	return &ShellV2Handler{}
}

func (h *ShellV2Handler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	conn, err := shellV2Upgrader.Upgrade(w, r, nil)
	if err != nil {
		fmt.Printf("ShellV2 Upgrade error: %v\n", err)
		return
	}
	defer conn.Close()

	fmt.Println("ShellV2 Client Connected")

	for {
		// Read message
		_, msg, err := conn.ReadMessage()
		if err != nil {
			if websocket.IsUnexpectedCloseError(err, websocket.CloseGoingAway, websocket.CloseAbnormalClosure) {
				fmt.Printf("ShellV2 Read error: %v\n", err)
			}
			break
		}

		fmt.Printf("ShellV2 Recv: %s\n", string(msg))

		// Parse request to extract command
		var req struct {
			Type    string `json:"type"`
			Command string `json:"command"`
		}
		if err := json.Unmarshal(msg, &req); err != nil {
			fmt.Printf("ShellV2 JSON Parse Error: %v\n", err)
			continue
		}

		if req.Type == "EXECUTE" {
			// Echo back
			resp := struct {
				Type    string `json:"type"`
				Content string `json:"content"`
			}{
				Type:    "OUTPUT",
				Content: fmt.Sprintf("Echo: %s\n", req.Command),
			}

			if err := conn.WriteJSON(resp); err != nil {
				fmt.Printf("ShellV2 Write error: %v\n", err)
				break
			}
		}
	}
}
