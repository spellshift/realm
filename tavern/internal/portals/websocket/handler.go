package websocket

import (
	"log/slog"
	"net/http"
	"strconv"

	"github.com/gorilla/websocket"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/internal/portals/websocket/ssh"
)

var upgrader = websocket.Upgrader{
	CheckOrigin: func(r *http.Request) bool { return true },
}

func NewPortalHandler(portalMux *mux.Mux) http.HandlerFunc {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		ctx := r.Context()

		// Parse portal ID
		portalIDStr := r.URL.Query().Get("portal_id")
		if portalIDStr == "" {
			http.Error(w, "missing 'portal_id' query parameter", http.StatusBadRequest)
			return
		}
		portalID, err := strconv.Atoi(portalIDStr)
		if err != nil {
			http.Error(w, "invalid 'portal_id' query parameter", http.StatusBadRequest)
			return
		}

		// Parse protocol
		proto := r.URL.Query().Get("proto")
		if proto == "" {
			http.Error(w, "missing 'proto' query parameter", http.StatusBadRequest)
			return
		}

		// Parse target
		target := r.URL.Query().Get("target")
		if target == "" {
			http.Error(w, "missing 'target' query parameter", http.StatusBadRequest)
			return
		}

		// Upgrade connection
		ws, err := upgrader.Upgrade(w, r, nil)
		if err != nil {
			slog.ErrorContext(ctx, "failed to upgrade portal websocket", "error", err)
			return
		}
		defer ws.Close()

		switch proto {
		case "ssh":
			if err := ssh.Handle(ctx, ws, portalMux, portalID, target); err != nil {
				slog.ErrorContext(ctx, "ssh portal handler failed", "error", err)
				ws.WriteMessage(websocket.CloseMessage, websocket.FormatCloseMessage(websocket.CloseInternalServerErr, err.Error()))
				return
			}
		default:
			slog.WarnContext(ctx, "unsupported portal protocol", "proto", proto)
			ws.WriteMessage(websocket.CloseMessage, websocket.FormatCloseMessage(websocket.CloseUnsupportedData, "unsupported protocol"))
			return
		}
	})
}
