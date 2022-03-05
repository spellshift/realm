package graphql_test

import (
	"context"
	"testing"

	"github.com/99designs/gqlgen/client"
	"github.com/99designs/gqlgen/graphql/handler"
	"github.com/kcarretto/realm/ent/enttest"
	"github.com/kcarretto/realm/graphql"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	_ "github.com/mattn/go-sqlite3"
)

// TestCreateImplantCallbackConfig ensures the createImplantCallbackConfig mutation exhibits expected behavior.
func TestCreateImplantCallbackConfig(t *testing.T) {
	// Initialize Test Context
	ctx := context.Background()

	// Initialize DB Backend
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	// Create a new GraphQL client
	gqlClient := client.New(handler.NewDefaultServer(graphql.NewSchema(graph)))

	// Run tests
	t.Run("WithDefaults", newCreateImplantCallbackConfigTest(
		gqlClient,
		graphql.CreateImplantCallbackConfigInput{
			URI: "https://withdefaults.realm.pub/",
		},
		func(t *testing.T, id int, err error) {
			require.NoError(t, err)
			assert.NotZero(t, id)

			cfg := graph.ImplantCallbackConfig.GetX(ctx, id)
			assert.Equal(t, "https://withdefaults.realm.pub/", cfg.URI)
			assert.Empty(t, cfg.ProxyURI)
			assert.NotZero(t, cfg.Priority)
			assert.NotZero(t, cfg.Timeout)
			assert.NotZero(t, cfg.Interval)
			assert.Zero(t, cfg.Jitter) // Default jitter is 0
		},
	))
	var (
		expectedProxyURI string = "https://proxy.uri:1337"
		expectedPriority int    = 1337
		expectedTimeout  int    = 54
		expectedInterval int    = 123
		expectedJitter   int    = 123
	)
	t.Run("WithValues", newCreateImplantCallbackConfigTest(
		gqlClient,
		graphql.CreateImplantCallbackConfigInput{
			URI:      "https://withvalues.realm.pub/",
			ProxyURI: &expectedProxyURI,
			Priority: &expectedPriority,
			Timeout:  &expectedTimeout,
			Interval: &expectedInterval,
			Jitter:   &expectedJitter,
		},
		func(t *testing.T, id int, err error) {
			require.NoError(t, err)
			assert.NotZero(t, id)

			cfg := graph.ImplantCallbackConfig.GetX(ctx, id)
			assert.Equal(t, "https://withvalues.realm.pub/", cfg.URI)
			assert.Equal(t, expectedProxyURI, cfg.ProxyURI)
			assert.Equal(t, expectedPriority, cfg.Priority)
			assert.Equal(t, expectedTimeout, cfg.Timeout)
			assert.Equal(t, expectedInterval, cfg.Interval)
			assert.Equal(t, expectedJitter, cfg.Jitter)
		},
	))
}

func newCreateImplantCallbackConfigTest(gqlClient *client.Client, input graphql.CreateImplantCallbackConfigInput, checks ...func(t *testing.T, id int, err error)) func(t *testing.T) {
	return func(t *testing.T) {
		// Define the mutatation for testing, taking the input as a variable
		mut := `mutation CreateImplantCallbackConfig($input: CreateImplantCallbackConfigInput!) { createImplantCallbackConfig(config:$input) { id } }`

		// Make our request to the GraphQL API
		var resp struct {
			CreateImplantCallbackConfig struct{ ID int }
		}
		err := gqlClient.Post(mut, &resp, client.Var("input", input))

		// Run checks with error (if any) and resulting ImplantCallbackConfig ID
		for _, check := range checks {
			check(t, resp.CreateImplantCallbackConfig.ID, err)
		}
	}
}

// TestUpdateImplantCallbackConfig ensures the updateImplantCallbackConfig mutation exhibits expected behavior.
func TestUpdateImplantCallbackConfig(t *testing.T) {
	// Initialize Test Context
	ctx := context.Background()

	// Initialize DB Backend
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	// Initialize sample data
	existingConfig := graph.ImplantCallbackConfig.Create().
		SetURI("https://existingconfig.realm.pub/").
		SaveX(ctx)

	// Create a new GraphQL client
	gqlClient := client.New(handler.NewDefaultServer(graphql.NewSchema(graph)))

	// Run tests
	t.Run("NoUpdate", newUpdateImplantCallbackConfigTest(
		gqlClient,
		graphql.UpdateImplantCallbackConfigInput{
			ID: existingConfig.ID,
		},
		func(t *testing.T, id int, err error) {
			require.NoError(t, err)
			assert.NotZero(t, id)
			assert.Equal(t, existingConfig.ID, id)

			cfg := graph.ImplantCallbackConfig.GetX(ctx, id)
			assert.Equal(t, existingConfig.URI, cfg.URI)
			assert.Equal(t, existingConfig.ProxyURI, cfg.ProxyURI)
			assert.Equal(t, existingConfig.Priority, cfg.Priority)
			assert.Equal(t, existingConfig.Timeout, cfg.Timeout)
			assert.Equal(t, existingConfig.Interval, cfg.Interval)
			assert.Equal(t, existingConfig.Jitter, cfg.Jitter)
		},
	))
	var (
		expectedURI      string = "https://updatedconfig.realm.pub/"
		expectedProxyURI string = "https://proxy.uri:1337"
		expectedPriority int    = 1337
		expectedTimeout  int    = 54
		expectedInterval int    = 123
		expectedJitter   int    = 123
	)
	t.Run("UpdateFields", newUpdateImplantCallbackConfigTest(
		gqlClient,
		graphql.UpdateImplantCallbackConfigInput{
			ID:       existingConfig.ID,
			URI:      &expectedURI,
			ProxyURI: &expectedProxyURI,
			Priority: &expectedPriority,
			Timeout:  &expectedTimeout,
			Interval: &expectedInterval,
			Jitter:   &expectedJitter,
		},
		func(t *testing.T, id int, err error) {
			require.NoError(t, err)
			assert.NotZero(t, id)
			assert.Equal(t, existingConfig.ID, id)

			cfg := graph.ImplantCallbackConfig.GetX(ctx, id)
			assert.Equal(t, expectedURI, cfg.URI)
			assert.Equal(t, expectedProxyURI, cfg.ProxyURI)
			assert.Equal(t, expectedPriority, cfg.Priority)
			assert.Equal(t, expectedTimeout, cfg.Timeout)
			assert.Equal(t, expectedInterval, cfg.Interval)
			assert.Equal(t, expectedJitter, cfg.Jitter)
		},
	))
	t.Run("NotExists", newUpdateImplantCallbackConfigTest(
		gqlClient,
		graphql.UpdateImplantCallbackConfigInput{
			ID: -1,
		},
		func(t *testing.T, id int, err error) {
			assert.Zero(t, id)
			assert.ErrorContains(t, err, "ent: implant_callback_config not found")
		},
	))
}

func newUpdateImplantCallbackConfigTest(gqlClient *client.Client, input graphql.UpdateImplantCallbackConfigInput, checks ...func(t *testing.T, id int, err error)) func(t *testing.T) {
	return func(t *testing.T) {
		// Define the mutatation for testing, taking the input as a variable
		mut := `mutation UpdateImplantCallbackConfig($input: UpdateImplantCallbackConfigInput!) { updateImplantCallbackConfig(config:$input) { id } }`

		// Make our request to the GraphQL API
		var resp struct {
			UpdateImplantCallbackConfig struct{ ID int }
		}
		err := gqlClient.Post(mut, &resp, client.Var("input", input))

		// Run checks with error (if any) and resulting ImplantCallbackConfig ID
		for _, check := range checks {
			check(t, resp.UpdateImplantCallbackConfig.ID, err)
		}
	}
}
