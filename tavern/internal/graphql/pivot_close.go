package graphql

import (
	"context"
	"log/slog"

	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/shellpivot"
	"realm.pub/tavern/internal/portals/pty"
	"realm.pub/tavern/internal/portals/ssh"
	"realm.pub/tavern/portals/portalpb"
)

func (r *mutationResolver) closeCachedPivotSession(ctx context.Context, pivot *ent.ShellPivot) {
	if pivot == nil {
		return
	}
	if pivot.Kind == shellpivot.KindSSH {
		ssh.ClosePivotSession(ctx, r.client, pivot.ID)
		return
	}
	pty.ClosePivotSession(ctx, r.client, pivot.ID)
}

func (r *mutationResolver) publishPivotCloseMote(ctx context.Context, pivot *ent.ShellPivot) {
	if pivot == nil || r.portalMux == nil || pivot.StreamID == "" {
		return
	}
	portalEnt := pivot.Edges.Portal
	if portalEnt == nil || !portalEnt.ClosedAt.IsZero() {
		return
	}

	closeMote := buildPivotCloseMote(pivot)
	if err := r.portalMux.Publish(ctx, r.portalMux.TopicIn(portalEnt.ID), closeMote); err != nil {
		slog.WarnContext(ctx, "failed to publish close mote for pivot",
			"pivot_id", pivot.ID,
			"portal_id", portalEnt.ID,
			"error", err,
		)
	}
}

func buildPivotCloseMote(pivot *ent.ShellPivot) *portalpb.Mote {
	if pivot.Kind == shellpivot.KindSSH {
		return &portalpb.Mote{
			StreamId: pivot.StreamID,
			Payload: &portalpb.Mote_Ssh{
				Ssh: &portalpb.SSHPayload{
					Kind: portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_CLOSE,
					Data: []byte("SSH session closed by operator"),
				},
			},
		}
	}
	return &portalpb.Mote{
		StreamId: pivot.StreamID,
		Payload: &portalpb.Mote_Bytes{
			Bytes: &portalpb.BytesPayload{
				Kind: portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE,
				Data: []byte("PTY session closed by operator"),
			},
		},
	}
}
