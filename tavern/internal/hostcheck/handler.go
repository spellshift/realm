package hostcheck

import (
	"crypto/ed25519"
	"net/http"
	"strings"
	"time"

	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/event"
	"realm.pub/tavern/internal/ent/host"
	"realm.pub/tavern/internal/logging"
)

var log = logging.For("scheduler.hostcheck")

// NewHandler returns an HTTP handler that checks all hosts for lost access.
// It queries the database for hosts whose NextSeenAt has been missed by more
// than 1 minute, and creates HOST_ACCESS_LOST events for those hosts.
// It will not create a duplicate event if a HOST_ACCESS_LOST event already
// exists for the host since its NextSeenAt time.
// The HookDeriveNotifications hook on the Event entity will automatically
// create URGENT notifications for each host's subscribers.
//
// The handler requires a valid JWT in the Authorization Bearer header, signed
// with the server's ed25519 key and scoped to the host-check audience.
func NewHandler(client *ent.Client, pubKey ed25519.PublicKey) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		traceID, reqID := logging.ExtractHTTPTrace(r.Header)
		ctx := logging.WithTrace(r.Context(), traceID, reqID)
		ctx = logging.WithEntrypoint(ctx, "scheduler")

		if r.Method != http.MethodPost {
			http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
			return
		}

		// Verify the JWT from the Authorization Bearer header.
		auth := strings.Fields(r.Header.Get("Authorization"))
		if len(auth) != 2 || !strings.EqualFold(auth[0], "Bearer") {
			http.Error(w, "missing token", http.StatusUnauthorized)
			return
		}
		if err := VerifyToken(auth[1], pubKey); err != nil {
			log.WarnContext(ctx, "invalid hostcheck token", "error", err)
			http.Error(w, "invalid token", http.StatusUnauthorized)
			return
		}

		now := time.Now()
		cutoff := now.Add(-1 * time.Minute)

		// Find all hosts where NextSeenAt is set and has been missed by more than 1 minute.
		hosts, err := client.Host.Query().
			Where(
				host.NextSeenAtNotNil(),
				host.NextSeenAtLT(cutoff),
			).
			All(ctx)
		if err != nil {
			log.ErrorContext(ctx, "failed to query hosts", "error", err)
			http.Error(w, "internal error", http.StatusInternalServerError)
			return
		}

		lostCount := 0
		for _, h := range hosts {
			// Check if a HOST_ACCESS_LOST event already exists for this host
			// since its NextSeenAt time. If so, we have already notified about
			// this particular loss of access and should not duplicate the event.
			// However, if the host recovers and loses access again (NextSeenAt
			// updates), a new event will be created because there won't be a
			// HOST_ACCESS_LOST event since the new NextSeenAt.
			nextSeenAtUnix := h.NextSeenAt.Unix()
			exists, err := client.Event.Query().
				Where(
					event.HasHostWith(host.ID(h.ID)),
					event.KindEQ(event.KindHOST_ACCESS_LOST),
					event.TimestampGTE(nextSeenAtUnix),
				).
				Exist(ctx)
			if err != nil {
				log.ErrorContext(ctx, "failed to query events", "error", err, "host_id", h.ID)
				continue
			}

			if exists {
				continue
			}

			// Host is lost — create the event.
			// HookDeriveNotifications will automatically create URGENT notifications.
			if err := client.Event.Create().
				SetKind(event.KindHOST_ACCESS_LOST).
				SetTimestamp(now.Unix()).
				SetHostID(h.ID).
				Exec(ctx); err != nil {
				log.ErrorContext(ctx, "failed to create HOST_ACCESS_LOST event", "error", err, "host_id", h.ID)
				continue
			}

			lostCount++
			log.InfoContext(ctx, "HOST_ACCESS_LOST event created", "host_id", h.ID, "host_name", h.Name, "host_identifier", h.Identifier)
		}

		if lostCount == 0 {
			log.DebugContext(ctx, "completed host access check", "overdue_hosts", len(hosts), "newly_lost_hosts", 0)
		}

		w.WriteHeader(http.StatusOK)
	})
}
