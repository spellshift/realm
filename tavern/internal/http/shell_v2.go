package http

import (
	"encoding/json"
	"fmt"
	"log/slog"
	"net/http"
	"strconv"
	"strings"
	"sync/atomic"

	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/shell"
	"realm.pub/tavern/internal/portals/mux"

	"github.com/gorilla/websocket"
)

var shellV2Upgrader = websocket.Upgrader{
	CheckOrigin: func(r *http.Request) bool {
		return true // Allow all origins for dev/testing
	},
}

type ShellV2Handler struct {
	graph *ent.Client
	mux   *mux.Mux
}

func NewShellV2Handler(graph *ent.Client, mux *mux.Mux) *ShellV2Handler {
	return &ShellV2Handler{
		graph: graph,
		mux:   mux,
	}
}

func (h *ShellV2Handler) parseShellIDFromRequest(r *http.Request) (int, error) {
	parts := strings.Split(r.URL.Path, "/")
	if len(parts) < 4 {
		return 0, fmt.Errorf("missing shell id")
	}
	shellIDStr := parts[len(parts)-1]
	shellID, err := strconv.Atoi(shellIDStr)
	if err != nil {
		return 0, fmt.Errorf("invalid shell id: %w", err)
	}
	return shellID, nil
}

func (h *ShellV2Handler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	shellID, err := h.parseShellIDFromRequest(r)
	if err != nil {
		http.Error(w, "invalid shell id", http.StatusBadRequest)
		return
	}

	ctx := r.Context()

	// Verify Shell Exists
	sh, err := h.graph.Shell.Query().
		Where(shell.ID(shellID)).
		WithBeacon().
		WithOwner().
		Only(ctx)
	if err != nil {
		if ent.IsNotFound(err) {
			http.Error(w, "shell not found", http.StatusNotFound)
			return
		}
		slog.ErrorContext(ctx, "failed to query shell", "error", err)
		http.Error(w, "internal server error", http.StatusInternalServerError)
		return
	}

	// Check if closed
	if !sh.ClosedAt.IsZero() {
		http.Error(w, "shell is closed", http.StatusGone)
		return
	}

	// Load Portal (if it exists)
	// TODO: We will want to start publishing to pubsub as soon as a portal exists, so we may need to periodically check to see if a portal has opened for this shell.
	// entPortal, err := sh.QueryPortals().Where(portal.ClosedAtIsNil()).First(ctx)
	// if err != nil && !ent.IsNotFound(err) {
	// 	slog.ErrorContext(ctx, "failed to query portal for shell", "error", err)
	// 	http.Error(w, "internal server error", http.StatusInternalServerError)
	// 	return
	// }
	// if entPortal != nil {
	// 	closePortal, err := h.mux.OpenPortal(ctx, entPortal.ID)
	// 	defer closePortal()
	// 	if err != nil {
	// 		slog.ErrorContext(ctx, "failed to open portal in mux", "error", err)
	// 		http.Error(w, "internal server error", http.StatusInternalServerError)
	// 		return
	// 	}
	// }

	// Upgrade to WebSocket
	conn, err := shellV2Upgrader.Upgrade(w, r, nil)
	if err != nil {
		slog.ErrorContext(ctx, "websocket upgrade failed", "error", err)
		return
	}
	defer conn.Close()
	slog.InfoContext(ctx, "shell client connected", "shell_id", shellID)

	// Determine current sequence ID (count existing tasks)
	count, err := sh.QueryShellTasks().Count(ctx)
	if err != nil {
		slog.ErrorContext(ctx, "failed to count shell tasks", "error", err)
		return
	}
	seqID := uint64(count)

	for {
		// Read message
		_, msg, err := conn.ReadMessage()
		if err != nil {
			if websocket.IsUnexpectedCloseError(err, websocket.CloseGoingAway, websocket.CloseAbnormalClosure) {
				slog.ErrorContext(ctx, "ShellV2 Read error", "error", err)
			}
			break
		}

		slog.DebugContext(ctx, "ShellV2 Recv", "msg", string(msg))

		// Parse request to extract command
		// Expected format: { "type": "EXECUTE", "command": "whoami" }
		var req struct {
			Type    string `json:"type"`
			Command string `json:"command"`
		}
		if err := json.Unmarshal(msg, &req); err != nil {
			slog.ErrorContext(ctx, "ShellV2 JSON Parse Error", "error", err)
			continue
		}

		slog.InfoContext(ctx, "shell request", "type", req.Type, "command", req.Command)
		if req.Type == "EXECUTE" {
			currentSeq := atomic.AddUint64(&seqID, 1)

			// 1. Write to DB
			_, err := h.graph.ShellTask.Create().
				SetShell(sh).
				SetInput(req.Command).
				SetSequenceID(currentSeq).
				Save(ctx)
			if err != nil {
				slog.ErrorContext(ctx, "failed to create shell task", "error", err)
				_ = conn.WriteJSON(map[string]string{
					"type":  "ERROR",
					"error": "Failed to persist task",
				})
				continue
			}
			if err := conn.WriteJSON(map[string]string{
				"type":    "OUTPUT",
				"content": fmt.Sprintf("[*] Task queued for %s \r\n", sh.Edges.Beacon.Name),
			}); err != nil {
				slog.ErrorContext(ctx, "failed to write shell task queued message", "error", err)
			}

			// TODO: If we have a portal open, use it to publish shell input and subscribe to shell output.
			// 2. Publish Mote (ShellPayload)
			// mote := &portalpb.Mote{
			// 	Payload: &portalpb.Mote_Shell{
			// 		Shell: &portalpb.ShellPayload{
			// 			Input:   req.Command,
			// 			ShellId: int64(shellID),
			// 		},
			// 	},
			// }
			// if err := h.mux.Publish(ctx, topicName, mote); err != nil {
			// 	slog.ErrorContext(ctx, "failed to publish shell input mote", "shell_id", shellID, "error", err)
			// 	_ = conn.WriteJSON(map[string]string{
			// 		"type":  "ERROR",
			// 		"error": "Failed to send command to agent",
			// 	})
			// }
		}
	}
}
