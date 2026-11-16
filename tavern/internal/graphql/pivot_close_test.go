package graphql_test

import (
	"context"
	"fmt"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/99designs/gqlgen/client"
	"github.com/99designs/gqlgen/graphql/handler"
	"github.com/gorilla/websocket"
	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/ent/shellpivot"
	"realm.pub/tavern/internal/graphql"
	tavernhttp "realm.pub/tavern/internal/http"
	"realm.pub/tavern/internal/portals/mux"
	"realm.pub/tavern/internal/portals/pty"
	"realm.pub/tavern/portals/portalpb"
	"realm.pub/tavern/tomes"
)

func TestClosePivot(t *testing.T) {
	ctx := context.Background()
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	portalMux := mux.New()
	git := tomes.NewGitImporter(graph)

	srv := tavernhttp.NewServer(
		tavernhttp.RouteMap{
			"/graphql": handler.NewDefaultServer(
				graphql.NewSchema(graph, git, graphql.WithPortalMux(portalMux)),
			),
		},
		tavernhttp.WithAuthenticationBypass(graph),
	)
	gqlClient := client.New(srv, client.Path("/graphql"))

	userEnt := graph.User.Create().
		SetOauthID("close-pivot-user").
		SetName("Close Pivot User").
		SetPhotoURL("http://example.com/photo.jpg").
		SetSessionToken("close-pivot-token").
		SetIsActivated(true).
		SaveX(ctx)

	hostEnt := graph.Host.Create().
		SetName("pivot-host").
		SetIdentifier("pivot-host-1").
		SetPrimaryIP("10.0.0.10").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		SaveX(ctx)

	beaconEnt := graph.Beacon.Create().
		SetIdentifier("test-beacon-01").
		SetName("test-beacon-01").
		SetHost(hostEnt).
		SetTransport(c2pb.Transport_TRANSPORT_HTTP1).
		SaveX(ctx)

	tomeEnt := graph.Tome.Create().
		SetName("pivot-tome").
		SetDescription("pivot tome").
		SetAuthor("test").
		SetEldritch("print('ok')").
		SaveX(ctx)

	questEnt := graph.Quest.Create().
		SetName("pivot-quest").
		SetCreator(userEnt).
		SetTome(tomeEnt).
		SaveX(ctx)

	taskEnt := graph.Task.Create().
		SetBeacon(beaconEnt).
		SetQuest(questEnt).
		SaveX(ctx)

	shellEnt := graph.Shell.Create().
		SetBeacon(beaconEnt).
		SetOwner(userEnt).
		SetData([]byte{}).
		SaveX(ctx)

	mutationQuery := `
		mutation ClosePivot($pivotID: ID!) {
			closePivot(pivotID: $pivotID) {
				id
				closedAt
			}
		}
	`

	t.Run("ClosesActivePTYPivotPublishesCloseMoteAndTearsDownWebsocket", func(t *testing.T) {
		portalID, agentCleanup, err := portalMux.CreatePortal(ctx, graph, taskEnt.ID, 0)
		require.NoError(t, err)
		defer agentCleanup()

		topicIn := portalMux.TopicIn(portalID)
		inCh, inCleanup := portalMux.Subscribe(topicIn)
		defer inCleanup()

		ptyHandler := pty.NewHandler(graph, portalMux)
		wsServer := httptest.NewServer(http.HandlerFunc(ptyHandler.ServeHTTP))
		defer wsServer.Close()

		wsURL := fmt.Sprintf("ws%s?portal_id=%d&shell_id=%d", strings.TrimPrefix(wsServer.URL, "http"), portalID, shellEnt.ID)
		ws, _, err := websocket.DefaultDialer.Dial(wsURL, nil)
		require.NoError(t, err)
		defer ws.Close()

		var ptyPivot *ent.ShellPivot
		require.Eventually(t, func() bool {
			pivots, qErr := graph.ShellPivot.Query().
				Where(shellpivot.KindEQ(shellpivot.KindPty)).
				All(ctx)
			if qErr != nil || len(pivots) == 0 {
				return false
			}
			ptyPivot = pivots[0]
			return true
		}, 2*time.Second, 20*time.Millisecond)

		// Drain initial PTY spawn mote
		select {
		case <-inCh:
		case <-time.After(2 * time.Second):
			t.Fatal("timed out waiting for initial PTY spawn mote")
		}

		var resp struct {
			ClosePivot struct {
				ID       string  `json:"id"`
				ClosedAt *string `json:"closedAt"`
			} `json:"closePivot"`
		}
		err = gqlClient.Post(mutationQuery, &resp, client.Var("pivotID", fmt.Sprintf("%d", ptyPivot.ID)))
		require.NoError(t, err)
		assert.Equal(t, fmt.Sprintf("%d", ptyPivot.ID), resp.ClosePivot.ID)
		require.NotNil(t, resp.ClosePivot.ClosedAt)

		// Expect close bytes mote published to TopicIn with matching stream ID
		sawCloseMote := false
		deadline := time.After(2 * time.Second)
		for !sawCloseMote {
			select {
			case m := <-inCh:
				if m.StreamId == ptyPivot.StreamID && m.GetBytes() != nil && m.GetBytes().Kind == portalpb.BytesPayloadKind_BYTES_PAYLOAD_KIND_CLOSE {
					sawCloseMote = true
				}
			case <-deadline:
				t.Fatal("timed out waiting for PTY close mote on TopicIn")
			}
		}

		reloaded, err := graph.ShellPivot.Get(ctx, ptyPivot.ID)
		require.NoError(t, err)
		assert.False(t, reloaded.ClosedAt.IsZero())
	})

	t.Run("ClosesActiveSSHPivotAndPublishesSSHCloseMote", func(t *testing.T) {
		portalID, agentCleanup, err := portalMux.CreatePortal(ctx, graph, taskEnt.ID, 0)
		require.NoError(t, err)
		defer agentCleanup()

		topicIn := portalMux.TopicIn(portalID)
		inCh, inCleanup := portalMux.Subscribe(topicIn)
		defer inCleanup()

		sshPivot := graph.ShellPivot.Create().
			SetShell(shellEnt).
			SetPortalID(portalID).
			SetStreamID("ssh-stream-close-test").
			SetKind(shellpivot.KindSSH).
			SetDestination("root@10.0.0.22:22").
			SetPort(22).
			SaveX(ctx)

		var resp struct {
			ClosePivot struct {
				ID       string  `json:"id"`
				ClosedAt *string `json:"closedAt"`
			} `json:"closePivot"`
		}
		err = gqlClient.Post(mutationQuery, &resp, client.Var("pivotID", fmt.Sprintf("%d", sshPivot.ID)))
		require.NoError(t, err)
		assert.Equal(t, fmt.Sprintf("%d", sshPivot.ID), resp.ClosePivot.ID)
		require.NotNil(t, resp.ClosePivot.ClosedAt)

		select {
		case m := <-inCh:
			assert.Equal(t, "ssh-stream-close-test", m.StreamId)
			require.NotNil(t, m.GetSsh())
			assert.Equal(t, portalpb.SSHPayloadKind_SSH_PAYLOAD_KIND_CLOSE, m.GetSsh().Kind)
		case <-time.After(2 * time.Second):
			t.Fatal("timed out waiting for SSH close mote on TopicIn")
		}

		reloaded, err := graph.ShellPivot.Get(ctx, sshPivot.ID)
		require.NoError(t, err)
		assert.False(t, reloaded.ClosedAt.IsZero())
	})

	t.Run("ClosesStalePivotWithoutOpenPortal", func(t *testing.T) {
		stalePortal := graph.Portal.Create().
			SetBeacon(beaconEnt).
			SetTask(taskEnt).
			SetOwner(userEnt).
			SetClosedAt(time.Now().Add(-5 * time.Minute)).
			SaveX(ctx)

		stalePivot := graph.ShellPivot.Create().
			SetShell(shellEnt).
			SetPortal(stalePortal).
			SetStreamID("ssh-stale-stream").
			SetKind(shellpivot.KindSSH).
			SetDestination("deploy@10.0.0.30:22").
			SetPort(22).
			SaveX(ctx)

		var resp struct {
			ClosePivot struct {
				ID       string  `json:"id"`
				ClosedAt *string `json:"closedAt"`
			} `json:"closePivot"`
		}
		err := gqlClient.Post(mutationQuery, &resp, client.Var("pivotID", fmt.Sprintf("%d", stalePivot.ID)))
		require.NoError(t, err)
		assert.Equal(t, fmt.Sprintf("%d", stalePivot.ID), resp.ClosePivot.ID)
		require.NotNil(t, resp.ClosePivot.ClosedAt)

		reloaded, err := graph.ShellPivot.Get(ctx, stalePivot.ID)
		require.NoError(t, err)
		assert.False(t, reloaded.ClosedAt.IsZero())
	})
}
