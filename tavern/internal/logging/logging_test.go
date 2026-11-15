package logging

import (
	"bytes"
	"context"
	"encoding/json"
	"log/slog"
	"net/http"
	"os"
	"path/filepath"
	"regexp"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc/metadata"
)

type capturePersister struct {
	mu      sync.Mutex
	entries []AuditEntry
}

func (c *capturePersister) Enqueue(e AuditEntry) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.entries = append(c.entries, e)
}

func (c *capturePersister) all() []AuditEntry {
	c.mu.Lock()
	defer c.mu.Unlock()
	out := make([]AuditEntry, len(c.entries))
	copy(out, c.entries)
	return out
}

func TestParseSpec_PresetsAndShorthand(t *testing.T) {
	t.Run("prod preset enables all request subsystems at INFO and defaults to JSON", func(t *testing.T) {
		spec, err := ParseSpec("prod")
		require.NoError(t, err)
		assert.Equal(t, PresetProd, spec.Preset)
		assert.Equal(t, FormatJSON, spec.DefaultFormat)
		assert.True(t, spec.IncludeInstanceID)
		assert.True(t, spec.LogGraphQLRawQuery)
		assert.Equal(t, slog.LevelInfo, spec.ResolveLevel("http.access"))
		assert.Equal(t, slog.LevelInfo, spec.ResolveLevel("graphql.query"))
		assert.Equal(t, slog.LevelInfo, spec.ResolveLevel("graphql.mutation"))
		assert.Equal(t, slog.LevelInfo, spec.ResolveLevel("grpc.access"))
		assert.Equal(t, slog.LevelInfo, spec.ResolveLevel("audit.quest"))
		assert.Equal(t, slog.LevelInfo, spec.ResolveLevel("portals.ssh"))
	})

	t.Run("dev preset suppresses routine ingress request chatter while keeping audits and ops at INFO", func(t *testing.T) {
		spec, err := ParseSpec("dev")
		require.NoError(t, err)
		assert.Equal(t, PresetDev, spec.Preset)
		assert.Equal(t, FormatText, spec.DefaultFormat)
		assert.False(t, spec.IncludeInstanceID)
		assert.False(t, spec.LogGraphQLRawQuery)
		assert.Equal(t, slog.LevelWarn, spec.ResolveLevel("http.access"))
		assert.Equal(t, slog.LevelWarn, spec.ResolveLevel("graphql.query"))
		assert.Equal(t, slog.LevelWarn, spec.ResolveLevel("graphql.mutation"))
		assert.Equal(t, slog.LevelWarn, spec.ResolveLevel("grpc.access"))
		assert.Equal(t, slog.LevelInfo, spec.ResolveLevel("audit.quest"))
		assert.Equal(t, slog.LevelInfo, spec.ResolveLevel("portals.ssh"))
	})

	t.Run("solo subsystem shorthand defaults unmatched subsystems to ERROR", func(t *testing.T) {
		for _, raw := range []string{"portals=debug", "portals"} {
			spec, err := ParseSpec(raw)
			require.NoError(t, err, "spec=%s", raw)
			assert.Equal(t, slog.LevelDebug, spec.ResolveLevel("portals"), "spec=%s", raw)
			assert.Equal(t, slog.LevelDebug, spec.ResolveLevel("portals.ssh"), "spec=%s", raw)
			assert.Equal(t, slog.LevelError, spec.ResolveLevel("graphql.query"), "spec=%s", raw)
			assert.Equal(t, slog.LevelError, spec.ResolveLevel("http.access"), "spec=%s", raw)
			assert.Equal(t, slog.LevelError, spec.ResolveLevel("c2.claim"), "spec=%s", raw)
			assert.Equal(t, slog.LevelError, spec.ResolveLevel("portalextra"), "spec=%s", raw) // respects dot boundary
		}
	})

	t.Run("bare dotted and multi-subsystem shorthands work with or without preset", func(t *testing.T) {
		spec, err := ParseSpec("portals.ssh,c2")
		require.NoError(t, err)
		assert.Equal(t, slog.LevelDebug, spec.ResolveLevel("portals.ssh"))
		assert.Equal(t, slog.LevelError, spec.ResolveLevel("portals.mux"))
		assert.Equal(t, slog.LevelDebug, spec.ResolveLevel("c2.claim"))
		assert.Equal(t, slog.LevelError, spec.ResolveLevel("graphql.query"))

		withPreset, err := ParseSpec("dev,portals")
		require.NoError(t, err)
		assert.Equal(t, PresetDev, withPreset.Preset)
		assert.Equal(t, slog.LevelDebug, withPreset.ResolveLevel("portals.mux"))
		assert.Equal(t, slog.LevelWarn, withPreset.ResolveLevel("graphql.query"))
		assert.Equal(t, slog.LevelInfo, withPreset.ResolveLevel("c2.claim"))
	})

	t.Run("composable prod plus subsystem override and longest prefix matching", func(t *testing.T) {
		spec, err := ParseSpec("prod,portals=debug,portals.ssh=trace")
		require.NoError(t, err)
		assert.Equal(t, slog.LevelInfo, spec.ResolveLevel("graphql.query"))
		assert.Equal(t, slog.LevelDebug, spec.ResolveLevel("portals.mux"))
		assert.Equal(t, LevelTrace, spec.ResolveLevel("portals.ssh"))
		assert.Equal(t, LevelTrace, spec.ResolveLevel("portals.ssh.channel"))
	})

	t.Run("off global level silences everything except selected subsystem", func(t *testing.T) {
		spec, err := ParseSpec("off,portals=debug")
		require.NoError(t, err)
		assert.Equal(t, LevelOff, spec.ResolveLevel("http.access"))
		assert.Equal(t, slog.LevelDebug, spec.ResolveLevel("portals.grpc"))
	})

	t.Run("rejects invalid directive", func(t *testing.T) {
		_, err := ParseSpec("portals=verbose")
		require.Error(t, err)
		_, err = ParseSpec("=debug")
		require.Error(t, err)
		_, err = ParseSpec("nonsense")
		require.Error(t, err)
		_, err = ParseSpec("portals..ssh")
		require.Error(t, err)
	})
}

func TestTraceExtractionAndPropagation(t *testing.T) {
	t.Run("extracts W3C traceparent first", func(t *testing.T) {
		h := make(http.Header)
		h.Set(HeaderTraceparent, "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01")
		h.Set(HeaderCloudTrace, "11111111111111111111111111111111/123;o=1")
		h.Set(HeaderRequestID, "req-custom-42")

		tid, rid := ExtractHTTPTrace(h)
		assert.Equal(t, "4bf92f3577b34da6a3ce929d0e0e4736", tid)
		assert.Equal(t, "req-custom-42", rid)
	})

	t.Run("falls back to X-Cloud-Trace-Context", func(t *testing.T) {
		h := make(http.Header)
		h.Set(HeaderCloudTrace, "105445aa7843bc8bf206b12000100000/1;o=1")

		tid, rid := ExtractHTTPTrace(h)
		assert.Equal(t, "105445aa7843bc8bf206b12000100000", tid)
		assert.Len(t, rid, 16)
	})

	t.Run("mints fresh IDs when headers missing or malformed", func(t *testing.T) {
		h := make(http.Header)
		h.Set(HeaderTraceparent, "00-00000000000000000000000000000000-0000000000000000-01")
		h.Set(HeaderRequestID, "bad\nheader\ninjection")

		tid, rid := ExtractHTTPTrace(h)
		assert.Len(t, tid, 32)
		assert.Len(t, rid, 16)
	})

	t.Run("roundtrips outgoing to incoming gRPC metadata", func(t *testing.T) {
		ctx := WithTrace(context.Background(), "abcdef0123456789abcdef0123456789", "0123456789abcdef")
		outCtx, tid, rid := InjectOutgoingGRPCTrace(ctx)
		assert.Equal(t, "abcdef0123456789abcdef0123456789", tid)
		assert.Equal(t, "0123456789abcdef", rid)

		outMD, ok := metadata.FromOutgoingContext(outCtx)
		require.True(t, ok)

		inCtx := metadata.NewIncomingContext(context.Background(), outMD)
		extractedCtx, gotTID, gotRID := ExtractGRPCTrace(inCtx)
		assert.Equal(t, tid, gotTID)
		assert.Equal(t, rid, gotRID)
		assert.Equal(t, tid, TraceIDFromContext(extractedCtx))
	})
}

func TestScrubbingNonMutating(t *testing.T) {
	orig := map[string]any{
		"input": map[string]any{
			"name":         "prod-ssh",
			"secret":       "super-secret-password",
			"credentialID": 12,
			"publicKey":    "ssh-ed25519 AAAA...",
		},
		"accessToken": "tok-123",
	}

	scrubbed := ScrubMap(orig)
	// Original map MUST remain untouched so GraphQL resolvers still receive real inputs!
	assert.Equal(t, "super-secret-password", orig["input"].(map[string]any)["secret"])
	assert.Equal(t, "tok-123", orig["accessToken"])

	// Scrubbed copy replaces secrets while keeping safe IDs and public keys.
	assert.Equal(t, RedactedPlaceholder, scrubbed["input"].(map[string]any)["secret"])
	assert.Equal(t, 12, scrubbed["input"].(map[string]any)["credentialID"])
	assert.Equal(t, "ssh-ed25519 AAAA...", scrubbed["input"].(map[string]any)["publicKey"])
	assert.Equal(t, RedactedPlaceholder, scrubbed["accessToken"])

	paramJSON := `{"cmd":"whoami","ssh_password":"hunter2","token_id":"99"}`
	scrubbedJSON := ScrubJSONString(paramJSON)
	assert.Contains(t, scrubbedJSON, `"cmd":"whoami"`)
	assert.Contains(t, scrubbedJSON, `"ssh_password":"[REDACTED]"`)
	assert.Contains(t, scrubbedJSON, `"token_id":"99"`)
}

func TestRouterFilteringAuditAndJSONSchema(t *testing.T) {
	var buf bytes.Buffer
	auditFile := filepath.Join(t.TempDir(), "audit.jsonl")
	persister := &capturePersister{}

	// Package-level logger created BEFORE Configure must pick up dynamic updates!
	portalLog := For("portals.ssh")
	gqlLog := For("graphql.query")

	noColor := false
	err := Configure(Options{
		Spec:          "portals=trace",
		Format:        FormatJSON,
		Output:        &buf,
		AuditFilePath: auditFile,
		InstanceID:    "tavern-test-1",
		IncludeInstanceID: func() *bool {
			b := true
			return &b
		}(),
		GCPProjectID: "realm-prod",
		Color:        &noColor,
		Persister:    persister,
	})
	require.NoError(t, err)
	t.Cleanup(func() {
		SetAuditPersister(nil)
		_ = Configure(Options{Spec: "dev"})
	})

	ctx := WithTrace(context.Background(), "4bf92f3577b34da6a3ce929d0e0e4736", "00f067aa0ba902b7")
	ctx = WithEntrypoint(ctx, "graphql")
	ctx = WithClientIP(ctx, "10.1.2.3")
	ctx = WithActor(ctx, Actor{
		Kind:       ActorUser,
		ID:         "7",
		Name:       "alice",
		IsAdmin:    true,
		AuthMethod: "cookie",
	})

	// Suppressed by solo-subsystem `portals=trace` (unmatched defaults to ERROR)
	gqlLog.InfoContext(ctx, "GetBeacons poll")

	// Allowed by `portals=trace`
	portalLog.TraceContext(ctx, "forwarding mote", "portal_id", 42, "secret", "dont-leak")

	// Audit record: console is `portals=trace` (so `audit.quest` at INFO is suppressed on console),
	// BUT BOTH `audit.jsonl` AND `AuditPersister` MUST still capture it!
	args := append(
		QuestAttrs(19, "enum-hosts", `{"cmd":"id","api_key":"secret123"}`),
		slog.Any("host_ids", []int{3, 4}),
	)
	Audit(ctx, "quest", "quest.created", "quest created", args...)

	lines := strings.Split(strings.TrimSpace(buf.String()), "\n")
	require.Len(t, lines, 1, "only portals.ssh log should appear on console")

	var consoleEntry map[string]any
	require.NoError(t, json.Unmarshal([]byte(lines[0]), &consoleEntry))
	assert.Equal(t, "DEBUG", consoleEntry["severity"])
	assert.Equal(t, "portals.ssh", consoleEntry["subsystem"])
	assert.Equal(t, "forwarding mote", consoleEntry["message"])
	assert.Equal(t, "tavern-test-1", consoleEntry["tavern_id"])
	assert.Equal(t, "4bf92f3577b34da6a3ce929d0e0e4736", consoleEntry["trace_id"])
	assert.Equal(t, "projects/realm-prod/traces/4bf92f3577b34da6a3ce929d0e0e4736", consoleEntry["logging.googleapis.com/trace"])
	assert.Equal(t, "00f067aa0ba902b7", consoleEntry["logging.googleapis.com/spanId"])
	assert.Equal(t, RedactedPlaceholder, consoleEntry["secret"])
	assert.Equal(t, "alice", consoleEntry["actor_name"])

	// Verify dedicated audit file captured the audit record with NOTICE severity.
	auditBytes, err := os.ReadFile(auditFile)
	require.NoError(t, err)
	var fileAudit map[string]any
	require.NoError(t, json.Unmarshal(bytes.TrimSpace(auditBytes), &fileAudit))
	assert.Equal(t, "NOTICE", fileAudit["severity"])
	assert.Equal(t, "audit", fileAudit["log_kind"])
	assert.Equal(t, "audit.quest", fileAudit["subsystem"])
	assert.Equal(t, "quest.created", fileAudit["audit_action"])
	assert.Equal(t, "success", fileAudit["audit_outcome"])

	// Verify AuditPersister received the structured AuditEntry with inferred target & scrubbed parameters.
	persisted := persister.all()
	require.Len(t, persisted, 1)
	assert.Equal(t, "quest.created", persisted[0].Action)
	assert.Equal(t, "quest", persisted[0].TargetType)
	assert.Equal(t, "19", persisted[0].TargetID)
	assert.Equal(t, "enum-hosts", persisted[0].TargetName)
	assert.Equal(t, "alice", persisted[0].Actor.Name)
	paramsMap, ok := persisted[0].Details["parameters"].(map[string]any)
	require.True(t, ok)
	assert.Equal(t, "id", paramsMap["cmd"])
	assert.Equal(t, RedactedPlaceholder, paramsMap["api_key"])
}

func TestScrubMap_NestedJSONStrings(t *testing.T) {
	rawParams := `{"greeting":"hello","api_key":"demo-secret-123"}`
	input := map[string]any{
		"input": map[string]any{
			"name":       "Lab Check",
			"parameters": rawParams,
		},
	}

	scrubbed := ScrubMap(input)
	nested, ok := scrubbed["input"].(map[string]any)
	require.True(t, ok)

	// Input must remain untouched
	assert.Equal(t, rawParams, input["input"].(map[string]any)["parameters"])

	var parsed map[string]any
	require.NoError(t, json.Unmarshal([]byte(nested["parameters"].(string)), &parsed))
	assert.Equal(t, "hello", parsed["greeting"])
	assert.Equal(t, RedactedPlaceholder, parsed["api_key"])
}

var testANSIPattern = regexp.MustCompile(`\x1b\[[0-9;]*m`)

func TestTextSinkAlignment(t *testing.T) {
	ts := time.Date(2026, 11, 15, 1, 18, 9, 823000000, time.UTC)

	for _, color := range []bool{false, true} {
		var buf bytes.Buffer
		sink := newTextSink(&buf, color)

		records := []recordEnvelope{
			{
				Subsystem: "app.startup",
				Record: func() slog.Record {
					r := slog.NewRecord(ts, slog.LevelInfo, "initializing scheduler", 0)
					r.AddAttrs(slog.String("uri", "mem://"))
					return r
				}(),
			},
			{
				Subsystem: "graphql.mutation",
				IsAudit:   false,
				Meta:      ContextMeta{TraceID: "4bf92f3577b34da6a3ce929d0e0e4736"},
				Record: func() slog.Record {
					r := slog.NewRecord(ts, slog.LevelWarn, "graphql operation failed", 0)
					r.AddAttrs(slog.String("operation", "UpdateUser"))
					return r
				}(),
			},
			{
				Subsystem: "c2",
				IsAudit:   true,
				Meta:      ContextMeta{TraceID: "abc"},
				Record: func() slog.Record {
					r := slog.NewRecord(ts, slog.LevelInfo, "beacon check", 0)
					r.AddAttrs(slog.Int("beacon_id", 7))
					return r
				}(),
			},
			{
				Subsystem: "portals.mux",
				Record:    slog.NewRecord(ts, slog.LevelDebug, "no attrs message", 0),
			},
			{
				Subsystem: "app.config",
				Record: func() slog.Record {
					r := slog.NewRecord(ts, slog.LevelInfo, "", 0)
					r.AddAttrs(slog.String("mode", "sqlite"))
					return r
				}(),
			},
			{
				Subsystem: "portals.grpc",
				Meta:      ContextMeta{TraceID: "f361d2ce111122223333444455556666"},
				Record: func() slog.Record {
					r := slog.NewRecord(ts, slog.LevelDebug, "this message is intentionally longer than forty characters", 0)
					r.AddAttrs(slog.Int("portal_id", 42))
					return r
				}(),
			},
			{
				Subsystem: "app.config",
				Record:    slog.NewRecord(ts, slog.LevelInfo, "", 0),
			},
		}

		for _, rec := range records {
			require.NoError(t, sink.WriteRecord(rec))
		}

		rawLines := strings.Split(strings.TrimSuffix(buf.String(), "\n"), "\n")
		require.Len(t, rawLines, len(records))

		lines := make([]string, len(rawLines))
		for i, raw := range rawLines {
			lines[i] = testANSIPattern.ReplaceAllString(raw, "")
			assert.Equal(t, lines[i], strings.TrimRight(lines[i], " "), "unexpected trailing whitespace on line %d (color=%v)", i, color)
		}

		// Body column aligns at index 34 for subsystems fitting within 16 chars (including brackets),
		// without reserving empty horizontal space for absent trace IDs.
		assert.Equal(t, 34, strings.Index(lines[0], "initializing scheduler uri=mem://"))
		assert.Equal(t, 34, strings.Index(lines[2], "beacon check beacon_id=7 trace=abc"))
		assert.Equal(t, 34, strings.Index(lines[3], "no attrs message"))
		assert.Equal(t, 34, strings.Index(lines[4], "mode=sqlite"))
		assert.Equal(t, 34, strings.Index(lines[5], "this message is intentionally longer"))

		// Subsystems exceeding 16 chars keep a single space separator before body.
		assert.Contains(t, lines[1], "[graphql.mutation] graphql operation failed operation=UpdateUser trace=4bf92f35")

		// No inflated multi-space gaps between message and attributes.
		assert.Contains(t, lines[5], "forty characters portal_id=42 trace=f361d2ce")

		// Empty record has no trailing spaces.
		assert.Equal(t, "01:18:09.823 INF [app.config]", lines[6])
	}
}
