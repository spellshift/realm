package ssh

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"log/slog"
	"net"
	"net/http"
	"strconv"
	"strings"
	"sync"
	"sync/atomic"
	"time"

	"github.com/gorilla/websocket"
	"realm.pub/tavern/internal/c2/epb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/beacon"
	enthost "realm.pub/tavern/internal/ent/host"
	"realm.pub/tavern/internal/ent/network"
	"realm.pub/tavern/internal/ent/networkhost"
	"realm.pub/tavern/internal/ent/portal"
	"realm.pub/tavern/internal/http/shell"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/portals/portalpb"
)

var upgrader = websocket.Upgrader{
	ReadBufferSize:  1024,
	WriteBufferSize: 1024,
	CheckOrigin:     func(r *http.Request) bool { return true },
}

// WebsocketResizeOrCloseMessage represents control messages (resize/close) from the client.
type WebsocketControlMessage struct {
	Kind  string `json:"kind"`
	Input string `json:"input,omitempty"`
	Rows  uint32 `json:"rows,omitempty"`
	Cols  uint32 `json:"cols,omitempty"`
}

// PivotSession represents an active SSH session that can be shared among multiple websockets.
// SSH protocol negotiation happens on the gRPC-receiving Tavern instance via SSHPayload motes.
type PivotSession struct {
	PivotID       int
	PortalID      int
	StreamID      string
	CancelContext context.CancelFunc
	seqID         uint64

	mu            sync.Mutex
	closed        bool
	connected     bool
	history       []byte
	dirty         bool
	activeSockets map[*websocket.Conn]chan interface{}
}

// sessionCache maintains a map of active PivotSessions.
var (
	cacheMu      sync.Mutex
	sessionCache = make(map[int]*PivotSession)
)

func (ps *PivotSession) nextSeqID() uint64 {
	return atomic.AddUint64(&ps.seqID, 1)
}

func (ps *PivotSession) MarkConnected() bool {
	ps.mu.Lock()
	defer ps.mu.Unlock()
	if ps.connected {
		return false
	}
	ps.connected = true
	return true
}

func (ps *PivotSession) Broadcast(msg interface{}) {
	ps.mu.Lock()
	defer ps.mu.Unlock()

	// Append to history if it's an output message
	if outMsg, ok := msg.(shell.WebsocketTaskOutputMessage); ok {
		ps.history = append(ps.history, []byte(outMsg.Output)...)
		ps.dirty = true
	} else if errMsg, ok := msg.(shell.WebsocketTaskErrorMessage); ok {
		ps.history = append(ps.history, []byte(errMsg.Error)...)
		ps.dirty = true
	} else if wsErr, ok := msg.(shell.WebsocketErrorMessage); ok {
		ps.history = append(ps.history, []byte(fmt.Sprintf("\r\n\x1b[31m[-] %s\x1b[0m\r\n", wsErr.Error))...)
		ps.dirty = true
	}

	for _, ch := range ps.activeSockets {
		select {
		case ch <- msg:
		default:
			// channel full, skip
		}
	}
}

func (ps *PivotSession) AddSocket(wsConn *websocket.Conn, outCh chan interface{}) {
	ps.mu.Lock()
	defer ps.mu.Unlock()

	ps.activeSockets[wsConn] = outCh

	status := "connecting"
	if ps.connected {
		status = "connected"
	}
	select {
	case outCh <- shell.WebsocketStatusMessage{
		Kind:    shell.WebsocketMessageKindStatus,
		Status:  status,
		PivotID: ps.PivotID,
	}:
	default:
	}

	// Send history to catch up
	if len(ps.history) > 0 {
		msg := shell.WebsocketTaskOutputMessage{
			Kind:   shell.WebsocketMessageKindOutput,
			Output: string(ps.history),
		}
		select {
		case outCh <- msg:
		default:
		}
	}
}

func (ps *PivotSession) RemoveSocket(wsConn *websocket.Conn) {
	ps.mu.Lock()
	defer ps.mu.Unlock()

	if ch, ok := ps.activeSockets[wsConn]; ok {
		close(ch)
	}
	delete(ps.activeSockets, wsConn)
}

func (ps *PivotSession) FlushHistory(ctx context.Context, graph *ent.Client) {
	ps.mu.Lock()
	dirty := ps.dirty
	history := make([]byte, len(ps.history))
	copy(history, ps.history)
	ps.dirty = false
	ps.mu.Unlock()

	if dirty {
		_ = graph.ShellPivot.UpdateOneID(ps.PivotID).SetData(string(history)).Exec(ctx)
	}
}

func (ps *PivotSession) Close(ctx context.Context, graph *ent.Client) {
	ps.mu.Lock()
	if ps.closed {
		ps.mu.Unlock()
		return
	}
	ps.closed = true
	ps.mu.Unlock()

	cacheMu.Lock()
	delete(sessionCache, ps.PivotID)
	cacheMu.Unlock()

	if ps.CancelContext != nil {
		ps.CancelContext()
	}

	// Final flush & close
	ps.FlushHistory(ctx, graph)
	_ = graph.ShellPivot.UpdateOneID(ps.PivotID).SetClosedAt(time.Now()).Exec(ctx)

	// Notify and close all active socket channels so writer goroutines flush before closing
	ps.mu.Lock()
	for _, ch := range ps.activeSockets {
		select {
		case ch <- shell.WebsocketStatusMessage{
			Kind:   shell.WebsocketMessageKindStatus,
			Status: "closed",
		}:
		default:
		}
		close(ch)
	}
	ps.activeSockets = make(map[*websocket.Conn]chan interface{})
	ps.mu.Unlock()
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

	// Upgrade to websocket first to send errors over ws
	wsConn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		slog.ErrorContext(ctx, "failed to upgrade ssh websocket", "error", err)
		return
	}
	defer wsConn.Close()

	// Helper to send errors over websocket
	sendWsError := func(errMsg string) {
		wsConn.WriteJSON(shell.WebsocketErrorMessage{
			Kind:  shell.WebsocketMessageKindError,
			Error: errMsg,
		})
	}

	// Parse Query Parameters
	pivotIDStr := r.URL.Query().Get("pivot_id")
	portalIDStr := r.URL.Query().Get("portal_id")
	target := r.URL.Query().Get("target")
	shellIDStr := r.URL.Query().Get("shell_id")
	credentialIDStr := r.URL.Query().Get("credential_id")

	var pivotSession *PivotSession

	if pivotIDStr != "" {
		// Reconnect to existing session
		pivotID, err := strconv.Atoi(pivotIDStr)
		if err != nil {
			sendWsError("invalid pivot_id")
			return
		}

		cacheMu.Lock()
		sess, ok := sessionCache[pivotID]
		cacheMu.Unlock()

		if !ok {
			// If session is no longer active in memory, serve any persisted history from DB and report closed status
			pivotEnt, err := shell.RetryOnLock(ctx, func() (*ent.ShellPivot, error) {
				return h.graph.ShellPivot.Get(ctx, pivotID)
			})
			if err == nil {
				if pivotEnt.Data != "" {
					_ = wsConn.WriteJSON(shell.WebsocketTaskOutputMessage{
						Kind:   shell.WebsocketMessageKindOutput,
						Output: pivotEnt.Data,
					})
				}
				if pivotEnt.ClosedAt.IsZero() {
					_ = h.graph.ShellPivot.UpdateOneID(pivotID).SetClosedAt(time.Now()).Exec(ctx)
				}
				_ = wsConn.WriteJSON(shell.WebsocketStatusMessage{
					Kind:   shell.WebsocketMessageKindStatus,
					Status: "closed",
				})
			} else {
				sendWsError("pivot session not found or expired")
			}
			_ = wsConn.WriteControl(
				websocket.CloseMessage,
				websocket.FormatCloseMessage(websocket.CloseNormalClosure, ""),
				time.Now().Add(time.Second),
			)
			_ = wsConn.SetReadDeadline(time.Now().Add(250 * time.Millisecond))
			_, _, _ = wsConn.ReadMessage()
			return
		}
		pivotSession = sess
	} else {
		// Create new session
		if portalIDStr == "" || target == "" {
			sendWsError("missing portal_id or target")
			return
		}

		portalID, err := strconv.Atoi(portalIDStr)
		if err != nil {
			sendWsError("invalid portal_id")
			return
		}

		// Require a shellID
		if shellIDStr == "" {
			sendWsError("missing shell_id")
			return
		}

		// Parse target: defaults to root@<target>:22
		user := "root"
		hasExplicitUser := false
		password := ""
		var privateKey []byte
		hostPortStr := target

		if atIdx := strings.LastIndex(target, "@"); atIdx != -1 {
			userPassStr := target[:atIdx]
			hostPortStr = target[atIdx+1:]

			userPass := strings.SplitN(userPassStr, ":", 2)
			if strings.TrimSpace(userPass[0]) != "" {
				user = strings.TrimSpace(userPass[0])
				hasExplicitUser = true
			}
			if len(userPass) == 2 {
				password = userPass[1]
			}
		}

		if credentialIDStr != "" {
			if credID, err := strconv.Atoi(credentialIDStr); err == nil && credID > 0 {
				cred, err := shell.RetryOnLock(ctx, func() (*ent.HostCredential, error) {
					return h.graph.HostCredential.Get(ctx, credID)
				})
				if err == nil && cred != nil {
					if !hasExplicitUser && strings.TrimSpace(cred.Principal) != "" {
						user = strings.TrimSpace(cred.Principal)
					}
					if cred.Kind == epb.Credential_KIND_SSH_KEY || strings.HasPrefix(strings.TrimSpace(cred.Secret), "-----BEGIN") {
						privateKey = []byte(cred.Secret)
					} else if password == "" {
						password = cred.Secret
					}
				}
			}
		}

		if len(privateKey) == 0 && strings.HasPrefix(strings.TrimSpace(password), "-----BEGIN") {
			privateKey = []byte(password)
			password = ""
		}

		host, portStr, err := net.SplitHostPort(hostPortStr)
		if err != nil {
			if strings.Contains(err.Error(), "missing port in address") || strings.Contains(err.Error(), "too many colons in address") {
				host = hostPortStr
				portStr = "22"
				hostPortStr = net.JoinHostPort(host, portStr)
			} else {
				sendWsError(fmt.Sprintf("invalid target host format: %v", err))
				return
			}
		} else {
			hostPortStr = net.JoinHostPort(host, portStr)
		}

		// Verify Portal (and upgrade to the beacon's newest open portal if a newer one exists)
		p, err := shell.RetryOnLock(ctx, func() (*ent.Portal, error) {
			return h.graph.Portal.Query().
				Where(portal.ID(portalID)).
				WithBeacon().
				Only(ctx)
		})
		if err != nil {
			sendWsError("portal not found")
			return
		}
		if p.Edges.Beacon != nil {
			latestPortal, latestErr := shell.RetryOnLock(ctx, func() (*ent.Portal, error) {
				return h.graph.Portal.Query().
					Where(
						portal.HasBeaconWith(beacon.ID(p.Edges.Beacon.ID)),
						portal.ClosedAtIsNil(),
					).
					Order(ent.Desc(portal.FieldID)).
					First(ctx)
			})
			if latestErr == nil && latestPortal != nil {
				p = latestPortal
				portalID = latestPortal.ID
			}
		}
		if !p.ClosedAt.IsZero() {
			sendWsError("portal is closed")
			return
		}

		// Open Portal Mux
		cleanupPortal, err := h.mux.OpenPortal(ctx, portalID)
		if err != nil {
			sendWsError(fmt.Sprintf("Failed to open portal: %v", err))
			return
		}

		topicIn := h.mux.TopicIn(portalID)
		topicOut := h.mux.TopicOut(portalID)

		// Stream ID
		streamID := fmt.Sprintf("ssh-ws-%d", time.Now().UnixNano())

		// Extract port as uint32
		dstPort, _ := strconv.ParseUint(portStr, 10, 32)
		if dstPort == 0 {
			dstPort = 22
		}

		// Create ShellPivot Ent
		pivotEnt, err := shell.RetryOnLock(ctx, func() (*ent.ShellPivot, error) {
			createReq := h.graph.ShellPivot.Create().
				SetStreamID(streamID).
				SetKind("ssh").
				SetDestination(hostPortStr).
				SetPort(int(dstPort)).
				SetPortalID(portalID)

			if shellIDStr != "" {
				if shellID, err := strconv.Atoi(shellIDStr); err == nil {
					createReq = createReq.SetShellID(shellID)
				}
			}

			return createReq.Save(ctx)
		})
		if err != nil {
			sendWsError(fmt.Sprintf("Failed to create shell pivot: %v", err))
			cleanupPortal()
			return
		}

		sessionCtx, cancelSession := context.WithCancel(context.Background())
		recvCh, recvCleanup := h.mux.Subscribe(topicOut)

		pivotSession = &PivotSession{
			PivotID:  pivotEnt.ID,
			PortalID: portalID,
			StreamID: streamID,
			CancelContext: func() {
				cancelSession()
				recvCleanup()
				cleanupPortal()
			},
			activeSockets: make(map[*websocket.Conn]chan interface{}),
		}

		cacheMu.Lock()
		sessionCache[pivotEnt.ID] = pivotSession
		cacheMu.Unlock()

		var closeOnce sync.Once
		closePivot := func() {
			closeOnce.Do(func() {
				pivotSession.Close(context.Background(), h.graph)
			})
		}

		// Background goroutine: read portal SSH output motes -> broadcast to websockets.
		go func() {
			for {
				select {
				case <-sessionCtx.Done():
					return
				case m, ok := <-recvCh:
					if !ok {
						closePivot()
						return
					}
					if m.StreamId != streamID {
						continue
					}
					if sshPayload := m.GetSsh(); sshPayload != nil {
						switch sshPayload.Kind {
						case portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_DATA:
							if pivotSession.MarkConnected() {
								recordSuccessfulPivotNetworkHost(sessionCtx, h.graph, portalID, host, int(dstPort))
								pivotSession.Broadcast(shell.WebsocketStatusMessage{
									Kind:    shell.WebsocketMessageKindStatus,
									Status:  "connected",
									PivotID: pivotSession.PivotID,
								})
							}
							if len(sshPayload.Data) > 0 {
								pivotSession.Broadcast(shell.WebsocketTaskOutputMessage{
									Kind:   shell.WebsocketMessageKindOutput,
									Output: string(sshPayload.Data),
								})
							}
						case portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_CLOSE:
							if len(sshPayload.Data) > 0 {
								pivotSession.Broadcast(shell.WebsocketErrorMessage{
									Kind:  shell.WebsocketMessageKindError,
									Error: string(sshPayload.Data),
								})
							}
							closePivot()
							return
						}
						continue
					}
					if m.GetBytes() != nil && m.GetBytes().Kind == portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE {
						closePivot()
						return
					}
				}
			}
		}()

		// Periodic history flush
		go func() {
			ticker := time.NewTicker(5 * time.Second)
			defer ticker.Stop()
			for {
				select {
				case <-sessionCtx.Done():
					return
				case <-ticker.C:
					pivotSession.FlushHistory(context.Background(), h.graph)
				}
			}
		}()

		// Attach current WS to PivotSession BEFORE publishing CONNECT so fast responses aren't missed
		outCh := make(chan interface{}, 64)
		pivotSession.AddSocket(wsConn, outCh)
		defer pivotSession.RemoveSocket(wsConn)

		// Publish SSH_PAYLOAD_KIND_CONNECT mote to instruct the gRPC-receiving Tavern instance
		// to initiate the SSH protocol over the portal stream.
		connectMote := &portalpb.Mote{
			StreamId: streamID,
			SeqId:    0,
			Payload: &portalpb.Mote_Ssh{
				Ssh: &portalpb.SSHPayload{
					Kind:       portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_CONNECT,
					DstAddr:    host,
					DstPort:    uint32(dstPort),
					Username:   user,
					Password:   password,
					PrivateKey: privateKey,
					Rows:       40,
					Cols:       80,
				},
			},
		}
		if err := h.mux.Publish(sessionCtx, topicIn, connectMote); err != nil {
			sendWsError(fmt.Sprintf("Failed to send SSH connect mote: %v", err))
			closePivot()
			return
		}

		h.serveSocketIO(ctx, wsConn, pivotSession, outCh, closePivot)
		return
	}

	// Attach current WS to existing PivotSession
	outCh := make(chan interface{}, 64)
	pivotSession.AddSocket(wsConn, outCh)
	defer pivotSession.RemoveSocket(wsConn)

	closePivot := func() {
		pivotSession.Close(context.Background(), h.graph)
	}

	h.serveSocketIO(ctx, wsConn, pivotSession, outCh, closePivot)
}

func (h *Handler) serveSocketIO(
	ctx context.Context,
	wsConn *websocket.Conn,
	pivotSession *PivotSession,
	outCh chan interface{},
	closePivot func(),
) {
	errCh := make(chan error, 2)
	topicIn := h.mux.TopicIn(pivotSession.PortalID)

	// Single writer goroutine for websocket
	go func() {
		for msg := range outCh {
			if err := wsConn.WriteJSON(msg); err != nil {
				errCh <- err
				return
			}
		}
		errCh <- io.EOF
	}()

	// WebSocket -> SSH portal motes
	go func() {
		for {
			_, msg, err := wsConn.ReadMessage()
			if err != nil {
				errCh <- err
				return
			}
			var ctrlMsg WebsocketControlMessage
			if err := json.Unmarshal(msg, &ctrlMsg); err != nil {
				continue
			}
			switch ctrlMsg.Kind {
			case shell.WebsocketMessageKindInput:
				mote := &portalpb.Mote{
					StreamId: pivotSession.StreamID,
					SeqId:    pivotSession.nextSeqID(),
					Payload: &portalpb.Mote_Ssh{
						Ssh: &portalpb.SSHPayload{
							Kind: portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_DATA,
							Data: []byte(ctrlMsg.Input),
						},
					},
				}
				if err := h.mux.Publish(ctx, topicIn, mote); err != nil {
					errCh <- err
					return
				}
			case "resize":
				if ctrlMsg.Rows > 0 && ctrlMsg.Cols > 0 {
					mote := &portalpb.Mote{
						StreamId: pivotSession.StreamID,
						SeqId:    pivotSession.nextSeqID(),
						Payload: &portalpb.Mote_Ssh{
							Ssh: &portalpb.SSHPayload{
								Kind: portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_RESIZE,
								Rows: ctrlMsg.Rows,
								Cols: ctrlMsg.Cols,
							},
						},
					}
					_ = h.mux.Publish(ctx, topicIn, mote)
				}
			case "close":
				mote := &portalpb.Mote{
					StreamId: pivotSession.StreamID,
					SeqId:    pivotSession.nextSeqID(),
					Payload: &portalpb.Mote_Ssh{
						Ssh: &portalpb.SSHPayload{
							Kind: portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_CLOSE,
						},
					},
				}
				_ = h.mux.Publish(ctx, topicIn, mote)
				closePivot()
				errCh <- io.EOF
				return
			}
		}
	}()

	// Wait for websocket to disconnect or session to close
	<-errCh
}

func deriveSubnetFromIP(ipStr string) string {
	parsedIP := net.ParseIP(strings.TrimSpace(ipStr))
	if parsedIP == nil {
		return ""
	}
	if ip4 := parsedIP.To4(); ip4 != nil {
		return fmt.Sprintf("%d.%d.%d.0/24", ip4[0], ip4[1], ip4[2])
	}
	mask := net.CIDRMask(64, 128)
	ipNet := &net.IPNet{IP: parsedIP.Mask(mask), Mask: mask}
	return ipNet.String()
}

func mergeUniquePort(ports []int, port int) []int {
	if port <= 0 {
		return ports
	}
	for _, existing := range ports {
		if existing == port {
			return ports
		}
	}
	return append(ports, port)
}

func findOrCreateNetwork(ctx context.Context, graph *ent.Client, subnet string, description string) (*ent.Network, error) {
	if subnet == "" {
		return nil, nil
	}
	existing, err := graph.Network.Query().
		Where(network.Subnet(subnet)).
		First(ctx)
	if err == nil {
		return existing, nil
	}
	if !ent.IsNotFound(err) {
		return nil, err
	}
	return graph.Network.Create().
		SetName(subnet).
		SetSubnet(subnet).
		SetDescription(description).
		Save(ctx)
}

func upsertNetworkHost(
	ctx context.Context,
	graph *ent.Client,
	name string,
	hostname string,
	ipAddr string,
	subnet string,
	openPort int,
	matchedHost *ent.Host,
	networks ...*ent.Network,
) (*ent.NetworkHost, error) {
	var existing *ent.NetworkHost
	var err error

	if matchedHost != nil {
		existing, err = graph.NetworkHost.Query().
			Where(networkhost.HasHostWith(enthost.ID(matchedHost.ID))).
			First(ctx)
	}
	if existing == nil && ipAddr != "" {
		existing, err = graph.NetworkHost.Query().
			Where(networkhost.IPAddress(ipAddr)).
			First(ctx)
	}
	if existing == nil && hostname != "" {
		existing, err = graph.NetworkHost.Query().
			Where(networkhost.Or(networkhost.Hostname(hostname), networkhost.Name(name))).
			First(ctx)
	}
	if err != nil && !ent.IsNotFound(err) {
		return nil, err
	}

	validNetworks := make([]*ent.Network, 0, len(networks))
	for _, n := range networks {
		if n != nil {
			validNetworks = append(validNetworks, n)
		}
	}

	if existing != nil {
		updater := existing.Update().
			SetOpenPorts(mergeUniquePort(existing.OpenPorts, openPort))
		if existing.Name == "" && name != "" {
			updater.SetName(name)
		}
		if existing.Hostname == "" && hostname != "" {
			updater.SetHostname(hostname)
		}
		if existing.IPAddress == "" && ipAddr != "" {
			updater.SetIPAddress(ipAddr)
		}
		if existing.Subnet == "" && subnet != "" {
			updater.SetSubnet(subnet)
		}
		if matchedHost != nil {
			updater.SetHost(matchedHost)
		}
		if len(validNetworks) > 0 {
			updater.AddNetworks(validNetworks...)
		}
		return updater.Save(ctx)
	}

	if existing == nil {
		if ipAddr == "" {
			if hostname != "" {
				ipAddr = hostname
			} else if name != "" {
				ipAddr = name
			}
		}
		if ipAddr == "" {
			return nil, nil
		}
	}

	creator := graph.NetworkHost.Create().
		SetName(name).
		SetHostname(hostname).
		SetIPAddress(ipAddr).
		SetSubnet(subnet).
		SetOpenPorts(mergeUniquePort([]int{}, openPort))
	if matchedHost != nil {
		creator.SetHost(matchedHost)
	}
	if len(validNetworks) > 0 {
		creator.AddNetworks(validNetworks...)
	}
	return creator.Save(ctx)
}

func recordSuccessfulPivotNetworkHost(ctx context.Context, graph *ent.Client, portalID int, targetHost string, targetPort int) {
	if graph == nil || targetHost == "" {
		return
	}

	var ipAddr string
	hostname := targetHost
	name := targetHost

	if parsedIP := net.ParseIP(targetHost); parsedIP != nil {
		ipAddr = parsedIP.String()
	}

	var matchedHost *ent.Host
	if ipAddr != "" {
		matchedHost, _ = graph.Host.Query().
			Where(enthost.Or(
				enthost.PrimaryIP(ipAddr),
				enthost.ExternalIP(ipAddr),
				enthost.Name(targetHost),
			)).
			First(ctx)
	} else {
		matchedHost, _ = graph.Host.Query().
			Where(enthost.Name(targetHost)).
			First(ctx)
	}

	if matchedHost != nil {
		if matchedHost.Name != "" {
			name = matchedHost.Name
			hostname = matchedHost.Name
		}
		if ipAddr == "" && matchedHost.PrimaryIP != "" {
			ipAddr = matchedHost.PrimaryIP
		}
	}

	subnet := deriveSubnetFromIP(ipAddr)
	targetNet, err := findOrCreateNetwork(
		ctx,
		graph,
		subnet,
		fmt.Sprintf("Discovered via SSH pivot to %s", targetHost),
	)
	if err != nil {
		slog.Error("failed to find or create target network for pivot", "target", targetHost, "error", err)
	}

	if _, err := upsertNetworkHost(ctx, graph, name, hostname, ipAddr, subnet, targetPort, matchedHost, targetNet); err != nil {
		slog.Error("failed to upsert target network host for pivot", "target", targetHost, "error", err)
	}

	srcHost, err := graph.Portal.Query().
		Where(portal.ID(portalID)).
		QueryBeacon().
		QueryHost().
		First(ctx)
	if err != nil || srcHost == nil {
		return
	}

	srcIP := strings.TrimSpace(srcHost.PrimaryIP)
	if srcIP == "" && srcHost.ExternalIP != "" && srcHost.ExternalIP != "NOOP" {
		srcIP = strings.TrimSpace(srcHost.ExternalIP)
	}
	if srcIP == "" {
		return
	}

	srcSubnet := deriveSubnetFromIP(srcIP)
	srcNet, err := findOrCreateNetwork(
		ctx,
		graph,
		srcSubnet,
		fmt.Sprintf("Network for host %s", srcHost.Name),
	)
	if err != nil {
		slog.Error("failed to find or create source network for pivot", "host", srcHost.Name, "error", err)
	}

	srcName := srcHost.Name
	if srcName == "" {
		srcName = srcIP
	}
	if srcName == "" {
		srcName = fmt.Sprintf("host-%d", srcHost.ID)
	}

	if _, err := upsertNetworkHost(ctx, graph, srcName, srcName, srcIP, srcSubnet, 0, srcHost, srcNet, targetNet); err != nil {
		slog.Error("failed to upsert source network host for pivot", "host", srcHost.Name, "error", err)
	}
}
