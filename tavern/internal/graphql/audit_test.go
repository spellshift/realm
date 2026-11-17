package graphql_test

import (
	"context"
	"encoding/json"
	"net/http"
	"strconv"
	"sync"
	"testing"
	"time"

	"github.com/99designs/gqlgen/client"
	"github.com/99designs/gqlgen/graphql/handler"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/graphql"
	tavernhttp "realm.pub/tavern/internal/http"
	"realm.pub/tavern/internal/logging"
	"realm.pub/tavern/tomes"
)

func sessionCookieOpt(token string) client.Option {
	return client.AddCookie(&http.Cookie{
		Name:    auth.SessionCookieName,
		Value:   token,
		Expires: time.Now().Add(24 * time.Hour),
	})
}

type gqlAuditRecorder struct {
	mu      sync.Mutex
	entries []logging.AuditEntry
}

func (r *gqlAuditRecorder) Enqueue(entry logging.AuditEntry) {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.entries = append(r.entries, entry)
}

func (r *gqlAuditRecorder) findOne(t *testing.T, action string) logging.AuditEntry {
	t.Helper()
	r.mu.Lock()
	defer r.mu.Unlock()
	var matches []logging.AuditEntry
	for _, entry := range r.entries {
		if entry.Action == action {
			matches = append(matches, entry)
		}
	}
	require.Len(t, matches, 1, "expected 1 audit entry for action %s", action)
	return matches[0]
}

func (r *gqlAuditRecorder) count() int {
	r.mu.Lock()
	defer r.mu.Unlock()
	return len(r.entries)
}

func marshalGQLDetails(t *testing.T, raw map[string]any) (string, map[string]any) {
	t.Helper()
	b, err := json.Marshal(raw)
	require.NoError(t, err)
	var out map[string]any
	require.NoError(t, json.Unmarshal(b, &out))
	return string(b), out
}

func TestMutationAuditEmissionAndScrubbing(t *testing.T) {
	ctx := context.Background()
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	rec := &gqlAuditRecorder{}
	logging.SetAuditPersister(rec)
	t.Cleanup(func() {
		logging.SetAuditPersister(nil)
	})

	git := tomes.NewGitImporter(graph)
	srv := tavernhttp.NewServer(
		tavernhttp.RouteMap{
			"/graphql": handler.NewDefaultServer(graphql.NewSchema(graph, git)),
		},
		tavernhttp.WithAuthentication(graph),
	)
	gqlClient := client.New(srv, client.Path("/graphql"))

	adminUser := graph.User.Create().
		SetName("audit-admin").
		SetOauthID("oauth-audit-admin").
		SetPhotoURL("https://example.com/audit-admin.png").
		SetSessionToken("audit-admin-session").
		SetAccessToken("initial-access-token").
		SetIsActivated(true).
		SetIsAdmin(true).
		SaveX(ctx)
	adminOpt := sessionCookieOpt(adminUser.SessionToken)

	testHost := graph.Host.Create().
		SetIdentifier("audit-host-1").
		SetName("audit-host").
		SetPlatform(c2pb.Host_PLATFORM_LINUX).
		SaveX(ctx)

	beacon1 := graph.Beacon.Create().
		SetHost(testHost).
		SetName("beacon-1").
		SetTransport(c2pb.Transport_TRANSPORT_HTTP1).
		SaveX(ctx)
	beacon2 := graph.Beacon.Create().
		SetHost(testHost).
		SetName("beacon-2").
		SetTransport(c2pb.Transport_TRANSPORT_HTTP1).
		SaveX(ctx)

	testTome := graph.Tome.Create().
		SetName("Credentialed Recon Tome").
		SetDescription("Tests quest parameter scrubbing").
		SetAuthor("tester").
		SetEldritch(`print("recon")`).
		SaveX(ctx)

	t.Run("CreateQuestScrubsSecrets", func(t *testing.T) {
		rawParams := `{"password":"super-secret-quest-pw","api_key":"tok-987654","cmd":"id -a"}`
		const mut = `mutation CreateAuditQuest($beaconIDs: [ID!]!, $input: CreateQuestInput!) {
			createQuest(beaconIDs: $beaconIDs, input: $input) {
				id
			}
		}`
		var resp struct {
			CreateQuest struct {
				ID string `json:"id"`
			} `json:"createQuest"`
		}
		err := gqlClient.Post(mut, &resp,
			adminOpt,
			client.Var("beaconIDs", []int{beacon1.ID, beacon2.ID}),
			client.Var("input", map[string]any{
				"name":       "AuditedQuest",
				"parameters": rawParams,
				"tomeID":     testTome.ID,
			}),
		)
		require.NoError(t, err)
		questID := convertID(resp.CreateQuest.ID)
		require.NotZero(t, questID)

		entry := rec.findOne(t, "quest.created")
		assert.Equal(t, "audit.quest", entry.Subsystem)
		assert.Equal(t, logging.OutcomeSuccess, entry.Outcome)
		assert.Equal(t, logging.ActorUser, entry.Actor.Kind)
		assert.Equal(t, strconv.Itoa(adminUser.ID), entry.Actor.ID)
		assert.Equal(t, adminUser.Name, entry.Actor.Name)
		assert.Equal(t, "quest", entry.TargetType)
		assert.Equal(t, strconv.Itoa(questID), entry.TargetID)
		assert.Equal(t, "AuditedQuest", entry.TargetName)

		detailsJSON, details := marshalGQLDetails(t, entry.Details)
		assert.NotContains(t, detailsJSON, "super-secret-quest-pw")
		assert.NotContains(t, detailsJSON, "tok-987654")
		assert.Equal(t, float64(questID), details["quest_id"])
		assert.Equal(t, float64(testTome.ID), details["tome_id"])
		assert.Equal(t, float64(2), details["task_count"])
		assert.Equal(t, []any{float64(testHost.ID)}, details["host_ids"])

		paramsMap, ok := details["parameters"].(map[string]any)
		require.True(t, ok, "expected scrubbed parameters object in audit details, got %T", details["parameters"])
		assert.Equal(t, logging.RedactedPlaceholder, paramsMap["password"])
		assert.Equal(t, logging.RedactedPlaceholder, paramsMap["api_key"])
		assert.Equal(t, "id -a", paramsMap["cmd"])
	})

	t.Run("CreateCredentialNeverLogsSecret", func(t *testing.T) {
		secretVal := "CLEARTEXT_NTLM_SECRET_DO_NOT_LOG"
		const mut = `mutation CreateAuditCred($input: CreateHostCredentialInput!) {
			createCredential(input: $input) {
				id
			}
		}`
		var resp struct {
			CreateCredential struct {
				ID string `json:"id"`
			} `json:"createCredential"`
		}
		err := gqlClient.Post(mut, &resp,
			adminOpt,
			client.Var("input", map[string]any{
				"principal": "CORP\\administrator",
				"secret":    secretVal,
				"kind":      "KIND_PASSWORD",
				"hostID":    testHost.ID,
			}),
		)
		require.NoError(t, err)
		credID := convertID(resp.CreateCredential.ID)
		require.NotZero(t, credID)

		entry := rec.findOne(t, "credential.created")
		assert.Equal(t, "audit.host", entry.Subsystem)
		assert.Equal(t, logging.OutcomeSuccess, entry.Outcome)
		assert.Equal(t, "credential", entry.TargetType)
		assert.Equal(t, strconv.Itoa(credID), entry.TargetID)
		assert.Equal(t, "CORP\\administrator", entry.TargetName)
		assert.NotContains(t, entry.Summary, secretVal)

		detailsJSON, details := marshalGQLDetails(t, entry.Details)
		assert.NotContains(t, detailsJSON, secretVal)
		assert.Equal(t, float64(credID), details["credential_id"])
		assert.Equal(t, float64(testHost.ID), details["host_id"])
		assert.Equal(t, "CORP\\administrator", details["principal"])
		assert.Equal(t, "KIND_PASSWORD", details["kind"])
		_, hasSecret := details["secret"]
		assert.False(t, hasSecret, "credential secret key must not be present in audit details")
	})

	t.Run("EnrollmentTokenAndAPIKeyNeverLogged", func(t *testing.T) {
		const enrollMut = `mutation { createBuilderEnrollmentToken }`
		var enrollResp struct {
			CreateBuilderEnrollmentToken string `json:"createBuilderEnrollmentToken"`
		}
		require.NoError(t, gqlClient.Post(enrollMut, &enrollResp, adminOpt))
		require.NotEmpty(t, enrollResp.CreateBuilderEnrollmentToken)

		const resetKeyMut = `mutation { resetUserAPIKey { id } }`
		var resetResp struct {
			ResetUserAPIKey struct {
				ID string `json:"id"`
			} `json:"resetUserAPIKey"`
		}
		require.NoError(t, gqlClient.Post(resetKeyMut, &resetResp, adminOpt))

		updatedAdmin := graph.User.GetX(ctx, adminUser.ID)
		require.NotEmpty(t, updatedAdmin.AccessToken)
		require.NotEqual(t, "initial-access-token", updatedAdmin.AccessToken)

		tokEntry := rec.findOne(t, "builder.enrollment_token_created")
		assert.Equal(t, "audit.admin", tokEntry.Subsystem)
		tokDetailsJSON, _ := marshalGQLDetails(t, tokEntry.Details)
		assert.NotContains(t, tokDetailsJSON, enrollResp.CreateBuilderEnrollmentToken)

		keyEntry := rec.findOne(t, "user.api_key_reset")
		assert.Equal(t, "audit.auth", keyEntry.Subsystem)
		keyDetailsJSON, _ := marshalGQLDetails(t, keyEntry.Details)
		assert.NotContains(t, keyDetailsJSON, updatedAdmin.AccessToken)
	})

	t.Run("DropAllDataEmitsAuditRecord", func(t *testing.T) {
		const dropMut = `mutation { dropAllData }`
		var dropResp struct {
			DropAllData bool `json:"dropAllData"`
		}
		require.NoError(t, gqlClient.Post(dropMut, &dropResp, adminOpt))
		assert.True(t, dropResp.DropAllData)

		// Engagement entities were wiped
		assert.Zero(t, graph.Host.Query().CountX(ctx))
		assert.Zero(t, graph.Beacon.Query().CountX(ctx))
		assert.Zero(t, graph.Quest.Query().CountX(ctx))
		assert.Zero(t, graph.HostCredential.Query().CountX(ctx))

		dropEntry := rec.findOne(t, "admin.drop_all_data")
		assert.Equal(t, "audit.admin", dropEntry.Subsystem)
		assert.Equal(t, logging.OutcomeSuccess, dropEntry.Outcome)
		assert.Equal(t, "dropped all engagement data", dropEntry.Summary)
		assert.Equal(t, strconv.Itoa(adminUser.ID), dropEntry.Actor.ID)
		assert.GreaterOrEqual(t, rec.count(), 5)
	})
}
