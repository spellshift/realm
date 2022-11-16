package graphql_test

import (
	"context"
	"testing"

	"github.com/99designs/gqlgen/client"
	"github.com/99designs/gqlgen/graphql/handler"
	"github.com/kcarretto/realm/tavern/ent/enttest"
	"github.com/kcarretto/realm/tavern/graphql"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestUsersQuery ensures that the users query works as expected.
func TestUsersQuery(t *testing.T) {
	// Initialize Test Context
	ctx := context.Background()

	// Initialize DB Backend
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	// Initialize sample data
	testUser := graph.User.Create().
		SetName("bobdylan").
		SetIsActivated(false).
		SetIsAdmin(true).
		SetOAuthID("likearollingstone").
		SetPhotoURL("https://upload.wikimedia.org/wikipedia/commons/0/02/Bob_Dylan_-_Azkena_Rock_Festival_2010_2.jpg").
		SaveX(ctx)

	// Create a new GraphQL server (needed for auth middleware)
	srv := handler.NewDefaultServer(graphql.NewSchema(graph))

	// Create a new GraphQL client (connected to our http server)
	gqlClient := client.New(srv)

	// Run Tests
	t.Run("QueryAll", newUsersQueryTest(
		gqlClient,
		func(t *testing.T, ids []int, err error) {
			require.NoError(t, err)
			require.NotEmpty(t, ids)
			assert.Len(t, ids, 1)
			user := graph.User.GetX(ctx, ids[0])
			assert.Equal(t, testUser.Name, user.Name)
			assert.Equal(t, testUser.IsActivated, user.IsActivated)
			assert.Equal(t, testUser.IsAdmin, user.IsAdmin)
		},
	))
}

func newUsersQueryTest(gqlClient *client.Client, checks ...func(t *testing.T, id []int, err error)) func(t *testing.T) {
	return func(t *testing.T) {
		// Define the query for testing
		query := `query { users { id } }`

		// Make our request to the GraphQL API
		var resp struct {
			Users []struct{ ID string }
		}
		err := gqlClient.Post(query, &resp)

		// Run checks with error (if any) and resulting id
		for _, check := range checks {
			ids := make([]int, 0, len(resp.Users))
			for _, user := range resp.Users {
				ids = append(ids, convertID(user.ID))
			}
			check(t, ids, err)
		}
	}
}
