package http

import (
	"context"
	"encoding/json"
	"fmt"
	"log/slog"
	"net/http"
	"strconv"
	"strings"
	"sync/atomic"
	"time"

	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/portal"
	"realm.pub/tavern/internal/ent/shell"
	"realm.pub/tavern/internal/ent/shelltask"
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

	// Use a cancellable context for internal coordination
	ctx, cancel := context.WithCancel(r.Context())
	defer cancel()

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

	// State
	var activePortalID int
	var portalCancel func()
	portalMsgCh := make(chan *portalpb.Mote, 100)
	sentBytes := make(map[uint64]int) // Map task_id (sequence_id) -> bytes sent

	// Channels
	wsReadCh := make(chan []byte)
	wsErrorCh := make(chan error, 1)
	wsWriteCh := make(chan interface{}, 100)

	// Timers
	portalCheckTicker := time.NewTicker(5 * time.Second)
	defer portalCheckTicker.Stop()

	dbPollTicker := time.NewTicker(1 * time.Second)
	defer dbPollTicker.Stop()

	// Helper to send to WS safely
	sendToWS := func(msg interface{}) {
		select {
		case wsWriteCh <- msg:
		case <-ctx.Done():
		}
	}

	// Start WS Reader
	go func() {
		for {
			_, msg, err := conn.ReadMessage()
			if err != nil {
				select {
				case wsErrorCh <- err:
				case <-ctx.Done():
				}
				return
			}
			select {
			case wsReadCh <- msg:
			case <-ctx.Done():
				return
			}
		}
	}()

	// Start WS Writer
	go func() {
		defer cancel() // Cancel context if writer exits
		for {
			select {
			case msg, ok := <-wsWriteCh:
				if !ok {
					return
				}
				if err := conn.WriteJSON(msg); err != nil {
					slog.ErrorContext(ctx, "ws write error", "error", err)
					return
				}
			case <-ctx.Done():
				return
			}
		}
	}()

	// Helper to cleanup portal sub
	cleanupPortal := func() {
		if portalCancel != nil {
			portalCancel()
			portalCancel = nil
		}
		activePortalID = 0
	}
	defer cleanupPortal()

	// Initial check for portal
	checkPortal := func() {
		entPortal, err := sh.QueryPortals().Where(portal.ClosedAtIsNil()).First(ctx)
		if err != nil && !ent.IsNotFound(err) {
			slog.ErrorContext(ctx, "failed to query portal for shell", "error", err)
			return
		}

		if entPortal == nil {
			if activePortalID != 0 {
				slog.InfoContext(ctx, "portal closed or lost", "old_portal_id", activePortalID)
				cleanupPortal()
			}
			return
		}

		if entPortal.ID != activePortalID {
			cleanupPortal()
			slog.InfoContext(ctx, "found new portal", "portal_id", entPortal.ID)
			activePortalID = entPortal.ID

			// Open portal in Mux (ensure topic exists)
			// Note: We subscribe to OUT topic
			topicOut := h.mux.TopicOut(activePortalID)
			subCh, cancel := h.mux.Subscribe(topicOut)
			portalCancel = cancel

			// Forward messages
			go func() {
				for {
					select {
					case mote, ok := <-subCh:
						if !ok {
							return
						}
						select {
						case portalMsgCh <- mote:
						case <-ctx.Done():
							return
						}
					case <-ctx.Done():
						return
					}
				}
			}()
		}
	}

	// Perform initial check
	checkPortal()

	for {
		select {
		case <-ctx.Done():
			return

		case err := <-wsErrorCh:
			if websocket.IsUnexpectedCloseError(err, websocket.CloseGoingAway, websocket.CloseAbnormalClosure) {
				slog.ErrorContext(ctx, "ShellV2 Read error", "error", err)
			}
			return

		case msg := <-wsReadCh:
			slog.DebugContext(ctx, "ShellV2 Recv", "msg", string(msg))

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

				// 1. Publish to Portal (if active)
				if activePortalID != 0 {
					topicIn := h.mux.TopicIn(activePortalID)
					mote := &portalpb.Mote{
						Payload: &portalpb.Mote_Shell{
							Shell: &portalpb.ShellPayload{
								Input:   req.Command,
								ShellId: int64(shellID),
								TaskId:  currentSeq,
							},
						},
					}
					if err := h.mux.Publish(ctx, topicIn, mote); err != nil {
						slog.ErrorContext(ctx, "failed to publish to portal", "error", err)
						// Fallback to just DB? Or error? We continue to DB write.
						// If publishing fails, the portal might be dead. Check again.
						checkPortal()
						if activePortalID == 0 {
							// Portal is gone, so we rely on DB.
						} else {
							// Portal still thinks it's up? Maybe just a glitch.
							cleanupPortal()
						}
					}
				}

				// 2. Write to DB
				_, err := h.graph.ShellTask.Create().
					SetShell(sh).
					SetInput(req.Command).
					SetSequenceID(currentSeq).
					Save(ctx)
				if err != nil {
					slog.ErrorContext(ctx, "failed to create shell task", "error", err)
					sendToWS(map[string]string{
						"type":  "ERROR",
						"error": "Failed to persist task",
					})
					continue
				}

				sendToWS(map[string]string{
					"type":    "OUTPUT",
					"content": fmt.Sprintf("[*] Task queued for %s \r\n", sh.Edges.Beacon.Name),
				})
			}

		case <-portalCheckTicker.C:
			checkPortal()

		case mote := <-portalMsgCh:
			// Process Portal Output
			if shellPayload, ok := mote.Payload.(*portalpb.Mote_Shell); ok {
				// Verify this is for the correct shell (optional, but good safety)
				if shellPayload.Shell.ShellId != int64(shellID) {
					continue
				}

				output := shellPayload.Shell.Output
				taskID := shellPayload.Shell.TaskId

				if output == "" {
					continue
				}

				// Deduplicate
				// Assumption: Portal sends CHUNKS.
				// We just send them and increment our counter.
				sendToWS(map[string]string{
					"type":    "OUTPUT",
					"content": output,
				})
				sentBytes[taskID] += len(output)
			}

		case <-dbPollTicker.C:
			// Poll DB for new output
			// We only care about tasks that have output we haven't fully sent.
			// Optimisation: only query tasks with sequence_id <= seqID (current)
			// For simplicity, we query the last few tasks or all tasks with output.
			// Since we track sentBytes, we can check.

			// We limit to recent tasks to avoid scanning full history every second
			// Query tasks with sequence_id > 0 (all valid tasks)
			// In production with pagination this needs care. Here we assume reasonable session size.
			tasks, err := sh.QueryShellTasks().
				Where(shelltask.SequenceIDGTE(0)).
				Order(ent.Asc(shelltask.FieldSequenceID)).
				All(ctx)

			if err != nil {
				slog.ErrorContext(ctx, "failed to poll shell tasks", "error", err)
				continue
			}

			for _, t := range tasks {
				fullOutput := t.Output
				sent := sentBytes[t.SequenceID]

				if len(fullOutput) > sent {
					newPart := fullOutput[sent:]
					sendToWS(map[string]string{
						"type":    "OUTPUT",
						"content": newPart,
					})
					sentBytes[t.SequenceID] = len(fullOutput)
				}
			}
		}
	}
}
