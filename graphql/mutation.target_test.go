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

// TestCreateTarget ensures the createTarget mutation exhibits expected behavior.
func TestCreateTarget(t *testing.T) {
	// Initialize Test Context
	ctx := context.Background()

	// Initialize DB Backend
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	// Create a new GraphQL client
	gqlClient := client.New(handler.NewDefaultServer(graphql.NewSchema(graph)))

	t.Run("New", newCreateTargetTest(
		gqlClient,
		graphql.CreateTargetInput{
			Name:             "TestTarget1",
			ForwardConnectIP: "10.0.0.1",
		},
		func(t *testing.T, id int, err error) {
			require.NoError(t, err)
			assert.NotZero(t, id)
			target := graph.Target.GetX(ctx, id)
			assert.Equal(t, "TestTarget1", target.Name)
			assert.Equal(t, "10.0.0.1", target.ForwardConnectIP)
		},
	))
	t.Run("DuplicateName", newCreateTargetTest(
		gqlClient,
		graphql.CreateTargetInput{
			Name:             "TestTarget1",
			ForwardConnectIP: "10.0.0.2",
		},
		func(t *testing.T, id int, err error) {
			assert.Zero(t, id)
			require.ErrorContains(t, err, "ent: constraint failed: UNIQUE constraint failed: targets.name")
		},
	))
	t.Run("DuplicateIP", newCreateTargetTest(
		gqlClient,
		graphql.CreateTargetInput{
			Name:             "TestTarget2",
			ForwardConnectIP: "10.0.0.1",
		},
		func(t *testing.T, id int, err error) {
			assert.Zero(t, id)
			require.ErrorContains(t, err, "ent: constraint failed: UNIQUE constraint failed: targets.forward_connect_ip")
		},
	))
}

func newCreateTargetTest(gqlClient *client.Client, input graphql.CreateTargetInput, checks ...func(t *testing.T, id int, err error)) func(t *testing.T) {
	return func(t *testing.T) {
		// Define the mutatation for testing, taking the input as a variable
		mut := `mutation CreateTarget($input: CreateTargetInput!) { createTarget(target:$input) { id } }`

		// Make our request to the GraphQL API
		var resp struct {
			CreateTarget struct{ ID int }
		}
		err := gqlClient.Post(mut, &resp, client.Var("input", input))

		// Run checks with error (if any) and resulting TargetID
		for _, check := range checks {
			check(t, resp.CreateTarget.ID, err)
		}
	}
}
