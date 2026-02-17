package shell

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"log/slog"
	"net/http"
	"strconv"
	"sync"
	"time"

	"entgo.io/ent/dialect/sql"
	"github.com/gorilla/websocket"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/portal"
	"realm.pub/tavern/internal/ent/shell"
	"realm.pub/tavern/internal/portals/mux"
)

const (
	defaultMaxInputsBuffered     = 1024 * 1024
	defaultMaxOutputsBuffered    = 1024 * 1024
	defaultMaxTaskErrorsBuffered = 1024
	defaultMaxErrorsBuffered     = 1024
	defaultPortalPollingInterval = 5 * time.Second
	defaultOutputPollingInterval = 1 * time.Second
	defaultKeepAliveInterval     = 1 * time.Second
	defaultWriteWaitTimeout      = 5 * time.Second
)

// A Handler for browser to server shell communication using websockets.
type Handler struct {
	graph *ent.Client
	mux   *mux.Mux

	maxInputsBuffered     int
	maxOutputsBuffered    int
	maxTaskErrorsBuffered int
	maxErrorsBuffered     int
	portalPollingInterval time.Duration
	outputPollingInterval time.Duration
	keepAliveInterval     time.Duration
	writeWaitTimeout      time.Duration
}

// NewHandler initializes and returns a new handler using the provided ent client and portal mux.
func NewHandler(graph *ent.Client, mux *mux.Mux) *Handler {
	return &Handler{
		graph: graph,
		mux:   mux,

		maxInputsBuffered:     defaultMaxInputsBuffered,
		maxOutputsBuffered:    defaultMaxOutputsBuffered,
		maxTaskErrorsBuffered: defaultMaxTaskErrorsBuffered,
		maxErrorsBuffered:     defaultMaxErrorsBuffered,
		portalPollingInterval: defaultPortalPollingInterval,
		outputPollingInterval: defaultOutputPollingInterval,
		keepAliveInterval:     defaultKeepAliveInterval,
		writeWaitTimeout:      defaultWriteWaitTimeout,
	}
}

// ServeHTTP will load prerequisite information and begin a websocket stream to connect to an agent shell.
// The handler supports both non-interactive and interactive modes of operation for the shell.
//
// Interactive Mode
//
//	If an open portal exists for the shell, the handler will leverage the portal mux to publish shell input
//	and receive shell output in real-time. Ent updates are performed out of band to provide low-latency
//	interactivity while maintaining state synchronization.
//
// Non-Interactive Mode
//
//	If no open portal exists for the shell, the handler will leverage ShellTask database ents for communication.
//	Output for these ents is polled, and periodic checks for an open portal are conducted. If an open portal is
//	found, the websocket will automatically switch to Interactive Mode. When the portal closes or if too many errors
//	occur the handler will automatically revert to Non-Interactive Mode and resume polling for open portals.
func (h *Handler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	// Load the shell
	sh, err := h.getShellForRequest(r)
	if err != nil && errors.Is(err, ErrShellIDInvalid) {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	if err != nil && errors.Is(err, ErrShellNotFound) {
		http.Error(w, err.Error(), http.StatusNotFound)
		return
	}
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	// Initialize Synchronization
	ctx, cancel := context.WithCancel(r.Context())
	var wg sync.WaitGroup
	wsTaskErrCh := make(chan *WebsocketTaskErrorMessage, h.maxTaskErrorsBuffered)
	wsErrCh := make(chan *WebsocketErrorMessage, h.maxErrorsBuffered)
	wsOutputCh := make(chan *WebsocketOutputMessage, h.maxOutputsBuffered)
	wsInputCh := make(chan *WebsocketInputMessage, h.maxInputsBuffered)
	wsPortalCh := make(chan *ent.Portal, 1)

	// Poll for open portals
	wg.Add(1)
	go func() {
		defer wg.Done()
		defer cancel()
		h.pollForOpenPortals(ctx, sh, wsPortalCh, wsErrCh)
	}()

	// Read messages
	wg.Add(1)
	go func() {
		defer wg.Done()
		defer cancel()
		h.readMessagesFromWebsocket(ctx, nil, wsInputCh, wsErrCh)
	}()

}

// getShellForRequest using the shell_id query param.
func (h *Handler) getShellForRequest(r *http.Request) (*ent.Shell, error) {
	// Parse Shell ID
	shellIDStr := r.URL.Query().Get("shell_id")
	if shellIDStr == "" {
		return nil, ErrShellIDInvalid
	}
	shellID, err := strconv.Atoi(shellIDStr)
	if err != nil {
		return nil, ErrShellIDInvalid
	}

	// Load Shell
	sh, err := h.graph.Shell.Query().
		Where(shell.ID(shellID)).
		Select(shell.FieldClosedAt).
		Only(r.Context())
	if err != nil {
		if ent.IsNotFound(err) {
			return nil, ErrShellNotFound
		}

		slog.ErrorContext(r.Context(), "websocket failed to load shell", "err", err, "shell_id", shellID)
		return nil, fmt.Errorf("%s: %w", ErrShellLookupFailed.Error(), err)
	}

	return sh, err
}

// pollForOpenPortals periodically checks for any open portals for the given shell.
func (h *Handler) pollForOpenPortals(ctx context.Context, sh *ent.Shell, portalCh chan<- *ent.Portal, errCh chan<- *WebsocketErrorMessage) {
	// Closure to poll a single time for open portals
	poll := func() {
		portal, err := sh.QueryPortals().
			Where(portal.ClosedAtIsNil()).
			Order(portal.ByCreatedAt(sql.OrderDesc())).
			First(ctx)
		if err != nil {
			errCh <- NewWebsocketErrorMessage(fmt.Errorf("%s: %w", ErrFailedToQueryPortals.Error(), err))
		}
		portalCh <- portal
	}

	// Periodically poll for open portals
	timer := time.NewTicker(h.portalPollingInterval)
	defer timer.Stop()
	for {
		select {
		case <-ctx.Done():
			return
		case <-timer.C:
			poll()
		}
	}
}

func (h *Handler) readMessagesFromWebsocket(ctx context.Context, conn *websocket.Conn, inputCh chan<- *WebsocketInputMessage, errCh chan<- *WebsocketErrorMessage) {
	// Read messages from the websocket, decode them, and send them to the input channel
	for {
		select {
		case <-ctx.Done():
			return
		default:
			// Read Message
			kind, msgData, err := conn.ReadMessage()
			if err != nil {
				if websocket.IsUnexpectedCloseError(err, websocket.CloseGoingAway, websocket.CloseAbnormalClosure) {
					slog.ErrorContext(ctx, "shell websocket closed unexpectedly",
						"error", err,
					)
				}
				return
			}
			if kind != websocket.BinaryMessage {
				continue
			}

			// Decode Message
			var msg WebsocketInputMessage
			if err := json.Unmarshal(msgData, &msg); err != nil {
				errCh <- NewWebsocketErrorMessage(fmt.Errorf("failed to decode message from client: %w", err))
			}

			// Send Message
			inputCh <- &msg
		}
	}
}

func (h *Handler) writeMessagesToWebsocket(ctx context.Context, conn *websocket.Conn, outputCh <-chan *WebsocketOutputMessage, taskErrCh <-chan *WebsocketOutputMessage, errCh <-chan *WebsocketErrorMessage, controlCh <-chan *WebsocketControlFlowMessage) {
	// Keep Alives
	keepAliveTimer := time.NewTicker(h.keepAliveInterval)
	defer keepAliveTimer.Stop()

	// Writes a single json message to the websocket
	writeJSONMessage := func(name string, v any, ok bool) error {
		// Set Deadline
		deadline := time.Now().Add(h.writeWaitTimeout)
		if err := conn.SetWriteDeadline(deadline); err != nil {
			return err
		}

		// Channel closed, cleanup
		if !ok {
			conn.WriteMessage(websocket.CloseMessage, []byte{})
			return fmt.Errorf("%s %w", name, ErrChannelClosed)
		}

		// Send Output
		if err := conn.WriteJSON(v); err != nil {
			return fmt.Errorf("failed to write message: %w", err)
		}

		return nil
	}

	// Write messages from channels to the websocket connection
	for {
		select {
		case <-ctx.Done():
			return
		case outputMsg, ok := <-outputCh:
			if err := writeJSONMessage("shell_output", outputMsg, ok); err != nil {
				slog.ErrorContext(ctx, "failed to write message to websocket", "error", err)
				return
			}
		case taskErrMsg, ok := <-taskErrCh:
			if err := writeJSONMessage("shell_task_errors", taskErrMsg, ok); err != nil {
				slog.ErrorContext(ctx, "failed to write message to websocket", "error", err)
				return
			}
		case errMsg, ok := <-errCh:
			if err := writeJSONMessage("shell_errors", errMsg, ok); err != nil {
				slog.ErrorContext(ctx, "failed to write message to websocket", "error", err)
				return
			}
		case <-keepAliveTimer.C:
			conn.WriteControl(websocket.PingMessage, []byte{})
			if err := conn.WriteMessage(websocket.PingMessage, nil); err != nil {
				return
			}
		}
	}
}
