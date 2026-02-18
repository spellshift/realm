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
	"github.com/google/uuid"
	"github.com/gorilla/websocket"
	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/portal"
	"realm.pub/tavern/internal/ent/shell"
	"realm.pub/tavern/internal/ent/shelltask"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/portals/portalpb"
)

const (
	defaultMaxTaskInputsBuffered   = 1024 * 1024
	defaultMaxTaskOutputsBuffered  = 1024 * 1024
	defaultMaxTaskErrorsBuffered   = 1024
	defaultMaxErrorsBuffered       = 1024
	defaultMaxControlFlowsBuffered = 1024
	defaultPortalPollingInterval   = 5 * time.Second
	defaultOutputPollingInterval   = 1 * time.Second
	defaultKeepAliveInterval       = 1 * time.Second
	defaultWriteWaitTimeout        = 5 * time.Second
)

// A Handler for browser to server shell communication using websockets.
type Handler struct {
	graph *ent.Client
	mux   *mux.Mux

	maxTaskInputsBuffered   int
	maxTaskOutputsBuffered  int
	maxTaskErrorsBuffered   int
	maxErrorsBuffered       int
	maxControlFlowsBuffered int
	portalPollingInterval   time.Duration
	outputPollingInterval   time.Duration
	keepAliveInterval       time.Duration
	writeWaitTimeout        time.Duration
}

// NewHandler initializes and returns a new handler using the provided ent client and portal mux.
func NewHandler(graph *ent.Client, mux *mux.Mux) *Handler {
	return &Handler{
		graph: graph,
		mux:   mux,

		maxTaskInputsBuffered:   defaultMaxTaskInputsBuffered,
		maxTaskOutputsBuffered:  defaultMaxTaskOutputsBuffered,
		maxTaskErrorsBuffered:   defaultMaxTaskErrorsBuffered,
		maxErrorsBuffered:       defaultMaxErrorsBuffered,
		maxControlFlowsBuffered: defaultMaxControlFlowsBuffered,
		portalPollingInterval:   defaultPortalPollingInterval,
		outputPollingInterval:   defaultOutputPollingInterval,
		keepAliveInterval:       defaultKeepAliveInterval,
		writeWaitTimeout:        defaultWriteWaitTimeout,
	}
}

// ShellSession holds the state for a single websocket connection.
type ShellSession struct {
	mu           sync.Mutex
	activePortal *ent.Portal
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
	// Load Authenticated User
	authUser := auth.UserFromContext(r.Context())
	var (
		authUserName = "unknown"
		authUserID   = 0
	)
	if authUser != nil {
		authUserID = authUser.ID
		authUserName = authUser.Name
	}
	slog.InfoContext(r.Context(), "starting new shell session", "user_id", authUserID, "user_name", authUserName)

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

	// Upgrade to websocket
	slog.InfoContext(r.Context(), "new shell websocket connection", "shell_id", sh.ID, "user_id", authUserID, "user_name", authUserName)
	wsConn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		slog.ErrorContext(r.Context(), "websocket failed to upgrade connection", "err", err, "shell_id", sh.ID)
		http.Error(w, "failed to upgrade to websocket", http.StatusFailedDependency)
		return
	}
	defer slog.InfoContext(r.Context(), "websocket shell connection closed", "shell_id", sh.ID, "user_id", authUserID, "user_name", authUserName)

	// Initialize Synchronization
	ctx, cancel := context.WithCancel(r.Context())
	var wg sync.WaitGroup
	wsTaskInputCh := make(chan *WebsocketTaskInputMessage, h.maxTaskInputsBuffered)
	wsTaskOutputCh := make(chan *WebsocketTaskOutputMessage, h.maxTaskOutputsBuffered)
	wsTaskOtherStreamCh := make(chan *WebsocketTaskOutputFromOtherStreamMessage, h.maxTaskOutputsBuffered)
	wsTaskErrCh := make(chan *WebsocketTaskErrorMessage, h.maxTaskErrorsBuffered)
	wsErrCh := make(chan *WebsocketErrorMessage, h.maxErrorsBuffered)
	wsControlFlowCh := make(chan *WebsocketControlFlowMessage, h.maxControlFlowsBuffered)
	portalCh := make(chan *ent.Portal, 1)
	streamID := uuid.NewString()

	session := &ShellSession{}

	// Poll for open portals
	wg.Add(1)
	go func() {
		defer wg.Done()
		defer cancel()
		h.pollForOpenPortals(ctx, sh, portalCh, wsErrCh)
	}()

	// Read messages
	wg.Add(1)
	go func() {
		defer wg.Done()
		defer cancel()
		h.readMessagesFromWebsocket(ctx, wsConn, wsTaskInputCh, wsErrCh)
	}()

	// Write messages
	wg.Add(1)
	go func() {
		defer wg.Done()
		defer cancel()
		h.writeMessagesToWebsocket(ctx, wsConn, wsTaskOutputCh, wsTaskOtherStreamCh, wsTaskErrCh, wsErrCh, wsControlFlowCh)
	}()

	// Publish shell input
	wg.Add(1)
	go func() {
		defer wg.Done()
		defer cancel()
		h.writeMessagesFromWebsocket(ctx, session, streamID, sh, wsTaskInputCh, wsControlFlowCh, wsErrCh)
	}()

	// Subscribe / Poll shell output
	wg.Add(1)
	go func() {
		defer wg.Done()
		defer cancel()
		h.writeMessagesFromShell(ctx, session, streamID, sh, portalCh, wsTaskOutputCh, wsTaskOtherStreamCh, wsTaskErrCh, wsControlFlowCh, wsErrCh)
	}()

	wg.Wait()
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
		WithBeacon().
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

// readMessagesFromWebsocket, decode them, and send them to the input channel.
func (h *Handler) readMessagesFromWebsocket(ctx context.Context, conn *websocket.Conn, inputCh chan<- *WebsocketTaskInputMessage, errCh chan<- *WebsocketErrorMessage) {
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
			if kind != websocket.BinaryMessage && kind != websocket.TextMessage {
				continue
			}

			// Decode Message
			var msg WebsocketTaskInputMessage
			if err := json.Unmarshal(msgData, &msg); err != nil {
				errCh <- NewWebsocketErrorMessage(fmt.Errorf("failed to decode message from client: %w", err))
			}

			// Send Message
			inputCh <- &msg
		}
	}
}

func (h *Handler) writeMessagesToWebsocket(ctx context.Context, conn *websocket.Conn, outputCh <-chan *WebsocketTaskOutputMessage, otherStreamCh <-chan *WebsocketTaskOutputFromOtherStreamMessage, taskErrCh <-chan *WebsocketTaskErrorMessage, errCh <-chan *WebsocketErrorMessage, controlCh <-chan *WebsocketControlFlowMessage) {
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
		case otherStreamMsg, ok := <-otherStreamCh:
			if err := writeJSONMessage("shell_output_other", otherStreamMsg, ok); err != nil {
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
		case controlMsg, ok := <-controlCh:
			if err := writeJSONMessage("shell_control", controlMsg, ok); err != nil {
				slog.ErrorContext(ctx, "failed to write message to websocket", "error", err)
			}
		case <-keepAliveTimer.C:
			conn.WriteControl(websocket.PingMessage, []byte{}, time.Now().Add(h.writeWaitTimeout))
			if err := conn.WriteMessage(websocket.PingMessage, nil); err != nil {
				continue
			}
		}
	}
}

// writeMessagesFromWebsocket receives input and control messages and writes them to the db / pubsub. If any errors occur, it writes them to the provided error channel.
func (h *Handler) writeMessagesFromWebsocket(ctx context.Context, session *ShellSession, streamID string, sh *ent.Shell, taskInputCh <-chan *WebsocketTaskInputMessage, controlCh chan<- *WebsocketControlFlowMessage, errCh chan<- *WebsocketErrorMessage) {
	var sequenceID uint64
	for {
		select {
		case <-ctx.Done():
			return
		case msg, ok := <-taskInputCh:
			if !ok {
				return
			}
			sequenceID++

			// Get Creator
			creator := auth.UserFromContext(ctx)
			if creator == nil {
				slog.ErrorContext(ctx, "no creator found in context")
				errCh <- NewWebsocketErrorMessage(fmt.Errorf("failed to determine creator for shell task"))
				continue
			}

			// Create ShellTask
			task, err := h.graph.ShellTask.Create().
				SetShell(sh).
				SetInput(msg.Input).
				SetCreator(creator).
				SetStreamID(streamID).
				SetSequenceID(sequenceID).
				Save(ctx)

			if err != nil {
				slog.ErrorContext(ctx, "failed to create shell task", "error", err)
				errCh <- NewWebsocketErrorMessage(fmt.Errorf("failed to create shell task: %w", err))
				continue
			}

			// Check for Active Portal
			session.mu.Lock()
			activePortal := session.activePortal
			session.mu.Unlock()

			if activePortal != nil {
				// Publish to Portal
				slog.InfoContext(ctx, "publishing to portal", "portal_id", activePortal.ID, "stream_id", streamID)
				mote := &portalpb.Mote{
					StreamId: streamID,
					SeqId:    sequenceID,
					Payload: &portalpb.Mote_Shell{
						Shell: &portalpb.ShellPayload{
							Input:    msg.Input,
							ShellId:  int64(sh.ID),
							StreamId: streamID,
						},
					},
				}
				if err := h.mux.Publish(ctx, h.mux.TopicIn(activePortal.ID), mote); err != nil {
					slog.ErrorContext(ctx, "failed to publish shell input mote", "error", err)
					// We don't error the task because it's already in DB, but maybe we should notify user?
				}
			} else {
				slog.InfoContext(ctx, "no active portal, queuing task", "shell_id", sh.ID)
				// No Active Portal: Send Control Flow Message
				beaconName := "Unknown"
				if sh.Edges.Beacon != nil {
					beaconName = sh.Edges.Beacon.Name
				} else {
					// Need to load beacon if not loaded.
					// We loaded it in ServeHTTP -> getShellForRequest -> WithBeacon()
					// But we passed `sh`. `sh` should have Edges populated.
					if sh.Edges.Beacon != nil {
						beaconName = sh.Edges.Beacon.Name
					}
				}

				controlCh <- &WebsocketControlFlowMessage{
					Kind:    WebsocketMessageKindControlFlow,
					Signal:  WebsocketControlFlowSignalTaskQueued,
					Message: fmt.Sprintf("[*] Task Queued for %s", beaconName),
				}
			}
			// We handle DB writes above (creation). Output writes happen in writeMessagesFromShell.
			_ = task
		}
	}
}

// writeMessagesFromShell receives output and task errors. It receives them from an open portal if it exists, otherwise it polls the db for any new messages.
// It caches output and errors that have already been sent to avoid duplication. If any errors occur, it writes them to the provided error channel.
func (h *Handler) writeMessagesFromShell(ctx context.Context, session *ShellSession, streamID string, sh *ent.Shell, portalCh <-chan *ent.Portal, taskOutputCh chan<- *WebsocketTaskOutputMessage, otherStreamCh chan<- *WebsocketTaskOutputFromOtherStreamMessage, taskErrCh chan<- *WebsocketTaskErrorMessage, controlCh chan<- *WebsocketControlFlowMessage, errCh <-chan *WebsocketErrorMessage) {
	var (
		cleanup     func()
		portalOutCh <-chan *portalpb.Mote
	)

	// Cache of sent task IDs to avoid duplication
	sentTasks := make(map[int]struct{})

	// Polls the DB for any new tasks with output
	pollForShellTasks := func() {
		tasks, err := sh.QueryShellTasks().
			Where(
				shelltask.Or(
					shelltask.OutputNEQ(""),
					shelltask.ErrorNEQ(""),
				),
			).
			WithCreator().
			All(ctx)

		if err != nil {
			slog.ErrorContext(ctx, "failed to poll shell tasks", "error", err)
			return
		}

		for _, task := range tasks {
			if _, sent := sentTasks[task.ID]; sent {
				continue
			}

			if task.Output != "" {
				taskOutputCh <- NewWebsocketTaskOutputMessage(task)
			}
			if task.Error != "" {
				taskErrCh <- NewWebsocketTaskErrorMessage(task)
			}
			sentTasks[task.ID] = struct{}{}
		}
	}

	pollTimer := time.NewTicker(h.outputPollingInterval)
	defer pollTimer.Stop()
	for {
		select {
		case <-ctx.Done():
			if cleanup != nil {
				cleanup()
			}
			return

		////
		// Interactive Mode (using portals)
		////
		case mote, ok := <-portalOutCh:
			// Handle Portal Closing
			if !ok {
				session.mu.Lock()
				portal := session.activePortal
				session.activePortal = nil
				session.mu.Unlock()

				// Inform browser that we've downgraded
				if portal != nil {
					controlCh <- NewWebsocketPortalDowngradeMessage(portal)
				}

				if cleanup != nil {
					cleanup()
				}
				cleanup = nil
				portalOutCh = nil // Stop receiving from closed channel
			}

			// Ignore empty motes
			if mote == nil {
				continue
			}

			// If stream_id matches, we send standard output
			// We check for bytes payload as it indicates output.
			bytesPayload := mote.GetBytes()
			if bytesPayload != nil {
				// It's output.
				task, err := h.graph.ShellTask.Query().
					Where(
						shelltask.StreamID(mote.StreamId),
						shelltask.SequenceID(mote.SeqId),
					).
					WithCreator().
					First(ctx)

				if err != nil {
					slog.DebugContext(ctx, "failed to find shell task for mote", "stream_id", mote.StreamId, "seq_id", mote.SeqId)
					continue
				}

				if mote.StreamId == streamID {
					// Local stream output
					outputMsg := NewWebsocketTaskOutputMessage(task)
					outputMsg.Output = string(bytesPayload.Data) // Use real-time chunk
					taskOutputCh <- outputMsg
					sentTasks[task.ID] = struct{}{}
				} else {
					// Other stream output
					if _, sent := sentTasks[task.ID]; !sent {
						otherStreamMsg := NewWebsocketTaskOutputFromOtherStreamMessage(task)
						// Custom formatting
						creatorName := "Unknown"
						if task.Edges.Creator != nil {
							creatorName = task.Edges.Creator.Name
						}

						truncatedInput := task.Input
						if len(truncatedInput) > 255 {
							truncatedInput = truncatedInput[:255] + "..."
						}

						otherStreamMsg.Output = fmt.Sprintf("[+] [%s] Task Output for %s\n", creatorName, truncatedInput)
						otherStreamMsg.Output += string(bytesPayload.Data)

						otherStreamCh <- otherStreamMsg
						sentTasks[task.ID] = struct{}{}
					} else {
						// Already sent header. Send additional data.
						otherStreamMsg := NewWebsocketTaskOutputFromOtherStreamMessage(task)
						otherStreamMsg.Output = string(bytesPayload.Data)
						otherStreamCh <- otherStreamMsg
					}
				}
			}

		////
		// Non-Interactive Mode (using db-polling)
		////
		case <-pollTimer.C:
			// Don't poll the DB if we have an active portal
			session.mu.Lock()
			hasActivePortal := session.activePortal != nil
			session.mu.Unlock()

			if hasActivePortal {
				continue
			}
			pollForShellTasks()

		////
		// Portal Upgrades
		////
		case p, ok := <-portalCh:
			// Portal polling channel has closed, exit gracefully
			if !ok {
				return
			}
			if p == nil {
				continue
			}

			// Open Portal
			portalCleanup, err := h.mux.OpenPortal(ctx, p.ID)
			if err != nil {
				slog.ErrorContext(ctx, "shell failed to upgrade with portal", "error", err, "portal_id", p.ID)
				continue
			}

			// Reconfigure to use portal
			var subCleanup func()
			portalOutCh, subCleanup = h.mux.Subscribe(h.mux.TopicOut(p.ID))

			// Chain cleanups
			cleanup = func() {
				if subCleanup != nil {
					subCleanup()
				}
				if portalCleanup != nil {
					portalCleanup()
				}
			}

			session.mu.Lock()
			session.activePortal = p
			session.mu.Unlock()

			// Inform browser that we've upgraded
			controlCh <- NewWebsocketPortalUpgradeMessage(p)
		}

	}
}
