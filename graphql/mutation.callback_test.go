package graphql_test

import (
	"context"
	"net/http"
	"testing"

	"github.com/kcarretto/realm/ent/enttest"
	"github.com/kcarretto/realm/ent/viewer"
	"github.com/kcarretto/realm/graphql"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/99designs/gqlgen/client"
	"github.com/99designs/gqlgen/graphql/handler"
	_ "github.com/mattn/go-sqlite3"
)

// TestCallback ensures the callback mutation exhibits expected behavior.
func TestCallback(t *testing.T) {
	// Initialize Test Context
	ctx := context.Background()

	// Initialize DB Backend
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	// Initialize sample data
	target := graph.Target.Create().
		SetName("Target1").
		SetForwardConnectIP("10.0.0.1").
		SaveX(ctx)
	impSvcConfig := graph.ImplantServiceConfig.Create().
		SetName("imix").
		SetDescription("a really cool implant!").
		SetExecutablePath("/bin/imix").
		SaveX(ctx)
	impCbConfig := graph.ImplantCallbackConfig.Create().
		SetURI("http://127.0.0.1:80/graphql").
		SaveX(ctx)
	impConfig := graph.ImplantConfig.Create().
		SetName("imix-rhel").
		SetAuthToken("supersecret!").
		AddServiceConfigs(impSvcConfig).
		AddCallbackConfigs(impCbConfig).
		SaveX(ctx)
	existingImplant := graph.Implant.Create().
		SetSessionID("AAAA-BBBB-CCCC").
		SetProcessName("/bin/imix").
		SetConfig(impConfig).
		SetTarget(target).
		SaveX(ctx)
	_ = existingImplant

	// Create a new GraphQL server (needed for auth middleware)
	srv := handler.NewDefaultServer(graphql.NewSchema(graph))
	httpSrv := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// Include our implant viewer in the context based on the X-Auth Header
		ctx := r.Context()
		authToken := r.Header.Get("X-Auth")
		if authToken != "" {
			ctx = viewer.NewContext(ctx, viewer.Implant{AuthToken: authToken})
			r = r.WithContext(ctx)
		}
		srv.ServeHTTP(w, r)
	})

	// Create a new GraphQL client (connected to our http server)
	gqlClient := client.New(httpSrv)
	t.Run("New", newCallbackTest(
		gqlClient,
		"supersecret!",
		graphql.CallbackInput{
			TargetID:    target.ID,
			SessionID:   "NewSessionID!",
			ConfigName:  impConfig.Name,
			ProcessName: "/lib/sodium",
		},
		func(t *testing.T, implantID int, err error) {
			require.NoError(t, err)
			assert.NotEqual(t, existingImplant.ID, implantID)
		},
	))
}

func newCallbackTest(gqlClient *client.Client, authToken string, input graphql.CallbackInput, checks ...func(t *testing.T, implantID int, err error)) func(t *testing.T) {
	return func(t *testing.T) {
		// Define the mutatation for testing, taking the input as a variable
		mut := `mutation callback($input: CallbackInput!) { callback(info:$input) { implant { id } } }`

		// Make our request to the GraphQL API
		var resp struct {
			Callback struct{ Implant struct{ ID int } }
		}
		err := gqlClient.Post(mut, &resp,
			client.Var("input", input),
			client.AddHeader("X-Auth", authToken),
		)

		// Run checks with error (if any) and resulting ImplantID
		for _, check := range checks {
			check(t, resp.Callback.Implant.ID, err)
		}
	}
}
