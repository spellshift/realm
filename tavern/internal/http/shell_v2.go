package http

import (
	"encoding/json"
	"fmt"
	"log/slog"
	"net/http"
	"strconv"
	"strings"
	"sync/atomic"

	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/shell"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/portals/portalpb"

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

func (h *ShellV2Handler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	// Extract shell_id from path
	// Assuming router passes pattern /shellv2/ws/{shellID}
	// Parsing shellID from URL path manually
	// URL: /shellv2/ws/<id>
	parts := strings.Split(r.URL.Path, "/")
	// /shellv2/ws/123 -> ["", "shellv2", "ws", "123"]
	if len(parts) < 4 {
		http.Error(w, "missing shell id", http.StatusBadRequest)
		return
	}
	shellIDStr := parts[len(parts)-1]
	shellID, err := strconv.Atoi(shellIDStr)
	if err != nil {
		http.Error(w, "invalid shell id", http.StatusBadRequest)
		return
	}

	ctx := r.Context()

	// Verify Shell Exists
	sh, err := h.graph.Shell.Query().
		Where(shell.ID(shellID)).
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

	// Authorization Check: Current User must be the Owner of the Shell
	currentUser := auth.UserFromContext(ctx)
	if currentUser == nil {
		http.Error(w, "unauthenticated", http.StatusUnauthorized)
		return
	}
	if sh.Edges.Owner == nil || sh.Edges.Owner.ID != currentUser.ID {
		// Also allow Admins? Requirement said "assert that the topic for the shell exists".
		// Usually only owner accesses their shell.
		// "If the shell with the provided id does not exist, it should return an error"
		// Secure by default: owner only.
		if !auth.IsAdminContext(ctx) {
			http.Error(w, "forbidden", http.StatusForbidden)
			return
		}
	}

	// Check if closed
	if !sh.ClosedAt.IsZero() {
		http.Error(w, "shell is closed", http.StatusGone)
		return
	}

	// Upgrade
	conn, err := shellV2Upgrader.Upgrade(w, r, nil)
	if err != nil {
		slog.ErrorContext(ctx, "ShellV2 Upgrade error", "error", err)
		return
	}
	defer conn.Close()

	slog.InfoContext(ctx, "ShellV2 Client Connected", "shell_id", shellID)

	// Topic for Shell Input (send to agent)
	topicName := fmt.Sprintf("SHELL_INPUT_%d", shellID)

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

			// 2. Publish Mote (ShellPayload)
			mote := &portalpb.Mote{
				Payload: &portalpb.Mote_Shell{
					Shell: &portalpb.ShellPayload{
						Input:   req.Command,
						ShellId: int64(shellID),
					},
				},
			}

			if err := h.mux.Publish(ctx, topicName, mote); err != nil {
				slog.ErrorContext(ctx, "failed to publish shell input mote", "shell_id", shellID, "error", err)
				_ = conn.WriteJSON(map[string]string{
					"type":  "ERROR",
					"error": "Failed to send command to agent",
				})
			}
		}
	}
}
