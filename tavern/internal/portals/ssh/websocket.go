package ssh

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"log/slog"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/gorilla/websocket"
	"golang.org/x/crypto/ssh"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/portal"
	"realm.pub/tavern/internal/http/shell"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/internal/portals/pnet"
	"realm.pub/tavern/portals/portalpb"
)

var upgrader = websocket.Upgrader{
	ReadBufferSize:  1024,
	WriteBufferSize: 1024,
	CheckOrigin:     func(r *http.Request) bool { return true },
}

// Handler handles websocket connections that tunnel SSH sessions over portal network.
type Handler struct {
	graph *ent.Client
	mux   *mux.Mux
}

// NewHandler creates a new SSH websocket handler.
func NewHandler(graph *ent.Client, mux *mux.Mux) *Handler {
	return &Handler{
		graph: graph,
		mux:   mux,
	}
}

func (h *Handler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()

	// Parse Query Parameters
	portalIDStr := r.URL.Query().Get("portal_id")
	target := r.URL.Query().Get("target")

	if portalIDStr == "" || target == "" {
		http.Error(w, "missing portal_id or target", http.StatusBadRequest)
		return
	}

	portalID, err := strconv.Atoi(portalIDStr)
	if err != nil {
		http.Error(w, "invalid portal_id", http.StatusBadRequest)
		return
	}

	// Parse target: user:password@host:port
	// We'll use a crude parse since the format is strict for now.
	var user, password, hostPort string
	parts := strings.SplitN(target, "@", 2)
	if len(parts) != 2 {
		http.Error(w, "invalid target format, expected user:password@host:port", http.StatusBadRequest)
		return
	}
	hostPort = parts[1]
	userPass := strings.SplitN(parts[0], ":", 2)
	user = userPass[0]
	if len(userPass) == 2 {
		password = userPass[1]
	}

	// Verify Portal
	p, err := h.graph.Portal.Query().Where(portal.ID(portalID)).Only(ctx)
	if err != nil {
		http.Error(w, "portal not found", http.StatusNotFound)
		return
	}
	if !p.ClosedAt.IsZero() {
		http.Error(w, "portal is closed", http.StatusForbidden)
		return
	}

	// Upgrade to websocket
	wsConn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		slog.ErrorContext(ctx, "failed to upgrade ssh websocket", "error", err)
		return
	}
	defer wsConn.Close()

	// Open Portal Mux
	cleanupPortal, err := h.mux.OpenPortal(ctx, portalID)
	if err != nil {
		wsConn.WriteJSON(shell.WebsocketErrorMessage{
			Kind:  shell.WebsocketMessageKindError,
			Error: fmt.Sprintf("Failed to open portal: %v", err),
		})
		return
	}
	defer cleanupPortal()

	topicIn := h.mux.TopicIn(portalID)
	topicOut := h.mux.TopicOut(portalID)

	// Stream ID
	streamID := fmt.Sprintf("ssh-ws-%d", time.Now().UnixNano())

	// pnet dial requirements
	sender := func(m *portalpb.Mote) error {
		return h.mux.Publish(context.Background(), topicIn, m)
	}

	recvCh, recvCleanup := h.mux.Subscribe(topicOut)
	defer recvCleanup()

	receiver := func() (*portalpb.Mote, error) {
		m, ok := <-recvCh
		if !ok {
			return nil, io.EOF
		}
		if m.GetBytes() != nil && m.GetBytes().Kind == portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE {
			return nil, io.EOF
		}
		return m, nil
	}

	// Dial pnet
	conn, err := pnet.Dial("tcp", hostPort, streamID, sender, receiver)
	if err != nil {
		wsConn.WriteJSON(shell.WebsocketErrorMessage{
			Kind:  shell.WebsocketMessageKindError,
			Error: fmt.Sprintf("Failed to dial portal network: %v", err),
		})
		return
	}
	defer conn.Close()

	// Setup SSH Client Config
	sshConfig := &ssh.ClientConfig{
		User:            user,
		Auth:            []ssh.AuthMethod{ssh.Password(password)},
		HostKeyCallback: ssh.InsecureIgnoreHostKey(),
		Timeout:         15 * time.Second,
	}

	// Connect SSH over pnet connection
	sshConnConn, chans, reqs, err := ssh.NewClientConn(conn, hostPort, sshConfig)
	if err != nil {
		wsConn.WriteJSON(shell.WebsocketErrorMessage{
			Kind:  shell.WebsocketMessageKindError,
			Error: fmt.Sprintf("Failed to establish SSH connection: %v", err),
		})
		return
	}
	sshClient := ssh.NewClient(sshConnConn, chans, reqs)
	defer sshClient.Close()

	session, err := sshClient.NewSession()
	if err != nil {
		wsConn.WriteJSON(shell.WebsocketErrorMessage{
			Kind:  shell.WebsocketMessageKindError,
			Error: fmt.Sprintf("Failed to open SSH session: %v", err),
		})
		return
	}
	defer session.Close()

	// Request PTY
	modes := ssh.TerminalModes{
		ssh.ECHO:          1,
		ssh.TTY_OP_ISPEED: 14400,
		ssh.TTY_OP_OSPEED: 14400,
	}
	if err := session.RequestPty("xterm", 80, 40, modes); err != nil {
		wsConn.WriteJSON(shell.WebsocketErrorMessage{
			Kind:  shell.WebsocketMessageKindError,
			Error: fmt.Sprintf("Failed to request PTY: %v", err),
		})
		return
	}

	stdin, err := session.StdinPipe()
	if err != nil {
		return
	}
	stdout, err := session.StdoutPipe()
	if err != nil {
		return
	}
	stderr, err := session.StderrPipe()
	if err != nil {
		return
	}

	if err := session.Shell(); err != nil {
		wsConn.WriteJSON(shell.WebsocketErrorMessage{
			Kind:  shell.WebsocketMessageKindError,
			Error: fmt.Sprintf("Failed to start shell: %v", err),
		})
		return
	}

	// Goroutines to proxy IO
	errCh := make(chan error, 4)
	outCh := make(chan interface{}, 32)

	// Single writer goroutine for websocket to prevent concurrent write panics
	go func() {
		for msg := range outCh {
			if err := wsConn.WriteJSON(msg); err != nil {
				errCh <- err
				return
			}
		}
	}()

	// WebSocket -> SSH Stdin
	go func() {
		for {
			_, msg, err := wsConn.ReadMessage()
			if err != nil {
				errCh <- err
				return
			}
			var inputMsg shell.WebsocketTaskInputMessage
			if err := json.Unmarshal(msg, &inputMsg); err == nil && inputMsg.Kind == shell.WebsocketMessageKindInput {
				stdin.Write([]byte(inputMsg.Input))
			}
		}
	}()

	// SSH Stdout -> WebSocket
	go func() {
		buf := make([]byte, 1024)
		for {
			n, err := stdout.Read(buf)
			if n > 0 {
				outCh <- shell.WebsocketTaskOutputMessage{
					Kind:   shell.WebsocketMessageKindOutput,
					Output: string(buf[:n]),
				}
			}
			if err != nil {
				errCh <- err
				return
			}
		}
	}()

	// SSH Stderr -> WebSocket
	go func() {
		buf := make([]byte, 1024)
		for {
			n, err := stderr.Read(buf)
			if n > 0 {
				outCh <- shell.WebsocketTaskErrorMessage{
					Kind:  shell.WebsocketMessageKindTaskError,
					Error: string(buf[:n]),
				}
			}
			if err != nil {
				errCh <- err
				return
			}
		}
	}()

	// Wait for session to finish or an error to occur
	go func() {
		errCh <- session.Wait()
	}()

	<-errCh
}
