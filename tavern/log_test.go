package main

import (
	"bytes"
	"context"
	"encoding/json"
	"log/slog"
	"net"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	gqlgraphql "github.com/99designs/gqlgen/graphql"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"github.com/urfave/cli"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/peer"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/logging"
)

func parseMainTestJSONLines(t *testing.T, buf *bytes.Buffer) []map[string]any {
	t.Helper()
	var out []map[string]any
	for _, line := range strings.Split(strings.TrimSpace(buf.String()), "\n") {
		if strings.TrimSpace(line) == "" {
			continue
		}
		var rec map[string]any
		require.NoError(t, json.Unmarshal([]byte(line), &rec))
		out = append(out, rec)
	}
	return out
}

func TestConfigureLoggingPrecedenceAndPresets(t *testing.T) {
	origDefault := DefaultLogSpec
	t.Cleanup(func() {
		DefaultLogSpec = origDefault
		_ = logging.Configure(logging.Options{Spec: "dev"})
	})

	t.Run("DefaultLogSpecDev", func(t *testing.T) {
		DefaultLogSpec = "dev"
		t.Setenv(EnvLogSpec.Key, "")
		t.Setenv(EnvLogFormat.Key, "")
		t.Setenv(EnvDebugLogging.Key, "")
		t.Setenv(EnvJSONLogging.Key, "")
		require.NoError(t, configureLogging("", ""))

		spec := logging.ActiveSpec()
		assert.Equal(t, logging.PresetDev, spec.Preset)
		assert.Equal(t, slog.LevelWarn, spec.ResolveLevel("http.access"))
		assert.Equal(t, slog.LevelInfo, spec.ResolveLevel("portals.ssh"))
		assert.False(t, logging.LogGraphQLRawQueryEnabled())
	})

	t.Run("LinkTimeProdDefault", func(t *testing.T) {
		DefaultLogSpec = "prod"
		t.Setenv(EnvLogSpec.Key, "")
		t.Setenv(EnvDebugLogging.Key, "")
		require.NoError(t, configureLogging("", ""))

		spec := logging.ActiveSpec()
		assert.Equal(t, logging.PresetProd, spec.Preset)
		assert.Equal(t, logging.FormatJSON, spec.DefaultFormat)
		assert.Equal(t, slog.LevelInfo, spec.ResolveLevel("http.access"))
		assert.True(t, logging.LogGraphQLRawQueryEnabled())
	})

	t.Run("IsolatedSubsystemFilter", func(t *testing.T) {
		DefaultLogSpec = "dev"
		for _, specStr := range []string{"portals=debug", "portals"} {
			require.NoError(t, configureLogging(specStr, ""), "spec=%s", specStr)

			spec := logging.ActiveSpec()
			assert.Equal(t, slog.LevelDebug, spec.ResolveLevel("portals"), "spec=%s", specStr)
			assert.Equal(t, slog.LevelDebug, spec.ResolveLevel("portals.ssh"), "spec=%s", specStr)
			assert.Equal(t, slog.LevelError, spec.ResolveLevel("c2"), "spec=%s", specStr)
			assert.Equal(t, slog.LevelError, spec.ResolveLevel("app.startup"), "spec=%s", specStr)
		}
	})

	t.Run("CLIWinsOverEnvAndLegacyEnvFallback", func(t *testing.T) {
		DefaultLogSpec = "dev"
		t.Setenv(EnvDebugLogging.Key, "1")
		t.Setenv(EnvLogSpec.Key, "")
		require.NoError(t, configureLogging("", ""))
		assert.Equal(t, slog.LevelDebug, logging.ActiveSpec().ResolveLevel("c2"))

		t.Setenv(EnvLogSpec.Key, "c2=warn")
		require.NoError(t, configureLogging("", ""))
		assert.Equal(t, slog.LevelWarn, logging.ActiveSpec().ResolveLevel("c2"))
		assert.Equal(t, slog.LevelError, logging.ActiveSpec().ResolveLevel("portals"))

		require.NoError(t, configureLogging("portals=trace", "json"))
		assert.Equal(t, logging.LevelTrace, logging.ActiveSpec().ResolveLevel("portals.mux"))
	})

	t.Run("SubcommandFlagsBeforeAndAfterCommandName", func(t *testing.T) {
		DefaultLogSpec = "dev"
		t.Setenv(EnvLogSpec.Key, "")

		var capturedSpec logging.Spec
		app := cli.NewApp()
		app.Flags = logFlags()
		app.Before = func(c *cli.Context) error {
			var s, f string
			if c.IsSet("log") {
				s = c.String("log")
			}
			if c.IsSet("log-format") {
				f = c.String("log-format")
			}
			return configureLogging(s, f)
		}
		app.Commands = []cli.Command{
			{
				Name:  "builder",
				Flags: logFlags(),
				Action: func(c *cli.Context) error {
					if err := applySubcommandLogFlags(c); err != nil {
						return err
					}
					capturedSpec = logging.ActiveSpec()
					return nil
				},
			},
		}

		require.NoError(t, app.Run([]string{"tavern", "builder", "--log=builder=debug"}))
		assert.Equal(t, slog.LevelDebug, capturedSpec.ResolveLevel("builder.exec"))
		assert.Equal(t, slog.LevelError, capturedSpec.ResolveLevel("c2"))

		require.NoError(t, app.Run([]string{"tavern", "--log=c2=debug", "builder"}))
		assert.Equal(t, slog.LevelDebug, capturedSpec.ResolveLevel("c2"))
	})
}

func TestGraphQLPostExecutionLoggingAndVariableRedaction(t *testing.T) {
	ctx := context.Background()
	client := enttest.OpenTempDB(t)
	defer client.Close()

	admin := client.User.Create().
		SetName("gql_admin").
		SetOauthID("gql_admin_oauth").
		SetPhotoURL("https://example.com/a.png").
		SetIsActivated(true).
		SetIsAdmin(true).
		SaveX(ctx)

	var buf bytes.Buffer
	logRaw := true
	require.NoError(t, logging.Configure(logging.Options{
		Spec:               "graphql=info",
		Format:             logging.FormatJSON,
		Output:             &buf,
		LogGraphQLRawQuery: &logRaw,
	}))
	t.Cleanup(func() {
		_ = logging.Configure(logging.Options{Spec: "dev"})
	})

	gqlHandler := newGraphQLHandler(client, nil)

	// Verify ScrubMap never mutates caller variable maps
	callerVars := map[string]any{
		"name":   "op-test",
		"secret": "my-raw-secret-value",
		"nested": map[string]any{
			"accessToken": "super-secret-pat",
			"hostID":      42,
		},
	}
	scrubbed := logging.ScrubMap(callerVars)
	assert.Equal(t, "my-raw-secret-value", callerVars["secret"])
	assert.Equal(t, "super-secret-pat", callerVars["nested"].(map[string]any)["accessToken"])
	assert.Equal(t, logging.RedactedPlaceholder, scrubbed["secret"])
	assert.Equal(t, logging.RedactedPlaceholder, scrubbed["nested"].(map[string]any)["accessToken"])

	// Execute a real GraphQL query with declared variables through newGraphQLHandler
	payload := map[string]any{
		"operationName": "UsersByFilter",
		"query":         "query UsersByFilter($tag: String!, $apiKey: String!) { users(where: { name: $tag, nameNEQ: $apiKey }) { totalCount } }",
		"variables": map[string]any{
			"apiKey": "do-not-leak-me",
			"tag":    "gql_admin",
		},
	}
	bodyBytes, err := json.Marshal(payload)
	require.NoError(t, err)

	req := httptest.NewRequest(http.MethodPost, "/graphql", bytes.NewReader(bodyBytes))
	req.Header.Set("Content-Type", "application/json")
	authCtx := auth.ContextFromUser(logging.WithTrace(req.Context(), "4bf92f3577b34da6a3ce929d0e0e4736", "00f067aa0ba902b7"), admin)
	req = req.WithContext(authCtx)

	w := httptest.NewRecorder()
	gqlHandler.ServeHTTP(w, req)
	require.Equal(t, http.StatusOK, w.Code, w.Body.String())
	assert.NotContains(t, buf.String(), "do-not-leak-me")

	// Also execute a GraphQL mutation that produces a resolver error -> graphql.mutation at WARNING
	mutPayload := map[string]any{
		"operationName": "UpdateMissingUser",
		"query":         "mutation UpdateMissingUser($id: ID!) { updateUser(userID: $id, input: { isActivated: true }) { id } }",
		"variables": map[string]any{
			"id": "999999",
		},
	}
	mutBytes, err := json.Marshal(mutPayload)
	require.NoError(t, err)
	mutReq := httptest.NewRequest(http.MethodPost, "/graphql", bytes.NewReader(mutBytes))
	mutReq.Header.Set("Content-Type", "application/json")
	mutReq = mutReq.WithContext(authCtx)
	mutW := httptest.NewRecorder()
	gqlHandler.ServeHTTP(mutW, mutReq)
	require.Equal(t, http.StatusOK, mutW.Code, mutW.Body.String())

	records := parseMainTestJSONLines(t, &buf)
	require.Len(t, records, 2)

	rec := records[0]
	assert.Equal(t, "graphql.query", rec["subsystem"])
	assert.Equal(t, "INFO", rec["severity"])
	assert.Equal(t, "graphql", rec["entrypoint"])
	assert.Equal(t, "UsersByFilter", rec["operation"])
	assert.Equal(t, "query", rec["operation_type"])
	assert.Equal(t, float64(0), rec["error_count"])
	assert.Contains(t, rec["raw_query"], "query UsersByFilter")
	varsMap, ok := rec["variables"].(map[string]any)
	require.True(t, ok)
	assert.Equal(t, logging.RedactedPlaceholder, varsMap["apiKey"])
	assert.Equal(t, "gql_admin", varsMap["tag"])

	mutRec := records[1]
	assert.Equal(t, "graphql.mutation", mutRec["subsystem"])
	assert.Equal(t, "WARNING", mutRec["severity"])
	assert.Equal(t, "UpdateMissingUser", mutRec["operation"])
	assert.Equal(t, "mutation", mutRec["operation_type"])
	assert.Equal(t, float64(1), mutRec["error_count"])
	_ = gqlgraphql.OperationContext{}
}

type fakeTransportStream struct {
	headerMD metadata.MD
}

func (f *fakeTransportStream) Method() string { return "/c2.C2/ClaimTasks" }
func (f *fakeTransportStream) SetHeader(md metadata.MD) error {
	f.headerMD = metadata.Join(f.headerMD, md)
	return nil
}
func (f *fakeTransportStream) SendHeader(md metadata.MD) error {
	f.headerMD = metadata.Join(f.headerMD, md)
	return nil
}
func (f *fakeTransportStream) SetTrailer(metadata.MD) error { return nil }

type fakeServerStream struct {
	grpc.ServerStream
	ctx      context.Context
	headerMD metadata.MD
}

func (f *fakeServerStream) Context() context.Context { return f.ctx }
func (f *fakeServerStream) SetHeader(md metadata.MD) error {
	f.headerMD = metadata.Join(f.headerMD, md)
	return nil
}

func TestGRPCUnaryAndStreamLoggingInterceptors(t *testing.T) {
	var buf bytes.Buffer
	require.NoError(t, logging.Configure(logging.Options{
		Spec:   "grpc.access=info",
		Format: logging.FormatJSON,
		Output: &buf,
	}))
	t.Cleanup(func() {
		_ = logging.Configure(logging.Options{Spec: "dev"})
	})

	incomingTrace := "4bf92f3577b34da6a3ce929d0e0e4736"
	md := metadata.Pairs("traceparent", "00-"+incomingTrace+"-00f067aa0ba902b7-01")
	baseCtx := metadata.NewIncomingContext(context.Background(), md)
	baseCtx = peer.NewContext(baseCtx, &peer.Peer{
		Addr: &net.TCPAddr{IP: net.ParseIP("203.0.113.9"), Port: 44123},
	})
	ts := &fakeTransportStream{}
	unaryCtx := grpc.NewContextWithServerTransportStream(baseCtx, ts)

	var handlerTrace, handlerEntrypoint, handlerIP string
	_, err := grpcWithUnaryLogging(
		unaryCtx,
		"req",
		&grpc.UnaryServerInfo{FullMethod: "/c2.C2/ClaimTasks"},
		func(ctx context.Context, _ any) (any, error) {
			handlerTrace = logging.TraceIDFromContext(ctx)
			handlerEntrypoint = logging.EntrypointFromContext(ctx)
			handlerIP = logging.ClientIPFromContext(ctx)
			return "ok", nil
		},
	)
	require.NoError(t, err)
	assert.Equal(t, incomingTrace, handlerTrace)
	assert.Equal(t, "grpc", handlerEntrypoint)
	assert.Equal(t, "203.0.113.9", handlerIP)
	assert.Equal(t, []string{incomingTrace}, ts.headerMD.Get(logging.MetadataTraceIDKey))
	assert.NotEmpty(t, ts.headerMD.Get(logging.MetadataReqIDKey))

	// Stream interceptor returning PermissionDenied -> WARNING
	ss := &fakeServerStream{ctx: baseCtx}
	err = grpcWithStreamLogging(
		nil,
		ss,
		&grpc.StreamServerInfo{FullMethod: "/portal.Portal/OpenPortal"},
		func(_ any, stream grpc.ServerStream) error {
			assert.Equal(t, incomingTrace, logging.TraceIDFromContext(stream.Context()))
			assert.Equal(t, "grpc", logging.EntrypointFromContext(stream.Context()))
			return status.Error(codes.PermissionDenied, "denied")
		},
	)
	require.Error(t, err)
	assert.Equal(t, []string{incomingTrace}, ss.headerMD.Get(logging.MetadataTraceIDKey))

	records := parseMainTestJSONLines(t, &buf)
	require.Len(t, records, 2)
	assert.Equal(t, "grpc.access", records[0]["subsystem"])
	assert.Equal(t, "INFO", records[0]["severity"])
	assert.Equal(t, "/c2.C2/ClaimTasks", records[0]["rpc_method"])
	assert.Equal(t, "OK", records[0]["grpc_code"])

	assert.Equal(t, "grpc.access", records[1]["subsystem"])
	assert.Equal(t, "WARNING", records[1]["severity"])
	assert.Equal(t, "/portal.Portal/OpenPortal", records[1]["rpc_method"])
	assert.Equal(t, "PermissionDenied", records[1]["grpc_code"])
}
