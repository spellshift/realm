package hostcheck

import (
	"encoding/json"
	"log/slog"
	"net/http"
	"time"

	"entgo.io/ent/dialect/sql"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/event"
	"realm.pub/tavern/internal/ent/host"
)

// Request is the JSON body sent to the host check endpoint.
type Request struct {
	HostID             int       `json:"host_id"`
	ExpectedNextSeenAt time.Time `json:"expected_next_seen_at"`
}

// NewHandler returns an HTTP handler that checks whether a host has been lost.
// If the host's NextSeenAt still matches the expected value (meaning no beacon
// has checked in since the check was scheduled), a HOST_ACCESS_LOST event is
// created. The HookDeriveNotifications hook on the Event entity will
// automatically create URGENT notifications for the host's subscribers.
func NewHandler(client *ent.Client) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}

		var req Request
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, "invalid request body", http.StatusBadRequest)
			return
		}

		ctx := r.Context()

		// Load the host
		h, err := client.Host.Get(ctx, req.HostID)
		if err != nil {
			if ent.IsNotFound(err) {
				http.Error(w, "host not found", http.StatusNotFound)
				return
			}
			slog.ErrorContext(ctx, "hostcheck: failed to load host", "err", err, "host_id", req.HostID)
			http.Error(w, "internal error", http.StatusInternalServerError)
			return
		}

		// If the host's NextSeenAt has changed since we scheduled this check,
		// a beacon has checked in and a newer check will handle this host.
		if !h.NextSeenAt.Equal(req.ExpectedNextSeenAt) {
			w.WriteHeader(http.StatusOK)
			return
		}

		// Check if a HOST_ACCESS_LOST event already exists for this host
		// without a subsequent HOST_ACCESS_RECOVERED event.
		latestEvent, err := client.Event.Query().
			Where(
				event.HasHostWith(host.ID(h.ID)),
				event.KindIn(event.KindHOST_ACCESS_LOST, event.KindHOST_ACCESS_RECOVERED),
			).
			Order(event.ByTimestamp(sql.OrderDesc())).
			First(ctx)
		if err != nil && !ent.IsNotFound(err) {
			slog.ErrorContext(ctx, "hostcheck: failed to query events", "err", err, "host_id", req.HostID)
			http.Error(w, "internal error", http.StatusInternalServerError)
			return
		}

		// If the latest event is already HOST_ACCESS_LOST, skip.
		if latestEvent != nil && latestEvent.Kind == event.KindHOST_ACCESS_LOST {
			w.WriteHeader(http.StatusOK)
			return
		}

		// Host is lost — create the event.
		// HookDeriveNotifications will automatically create URGENT notifications.
		if err := client.Event.Create().
			SetKind(event.KindHOST_ACCESS_LOST).
			SetTimestamp(time.Now().Unix()).
			SetHostID(h.ID).
			Exec(ctx); err != nil {
			slog.ErrorContext(ctx, "hostcheck: failed to create HOST_ACCESS_LOST event", "err", err, "host_id", req.HostID)
			http.Error(w, "internal error", http.StatusInternalServerError)
			return
		}

		slog.InfoContext(ctx, "hostcheck: HOST_ACCESS_LOST event created", "host_id", req.HostID, "host_identifier", h.Identifier)
		w.WriteHeader(http.StatusOK)
	})
}
