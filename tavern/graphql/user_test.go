package graphql_test

import (
	"context"
	"testing"

	"github.com/kcarretto/realm/tavern/ent"
	"github.com/kcarretto/realm/tavern/ent/enttest"
	"github.com/kcarretto/realm/tavern/graphql"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/99designs/gqlgen/client"
	"github.com/99designs/gqlgen/graphql/handler"
	_ "github.com/mattn/go-sqlite3"
)

// TestUserMutations ensures the user mutations exhibits expected behavior.
func TestUserMutations(t *testing.T) {
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

	newName := "bobbyd"
	t.Run("Update", newUpdateUserTest(
		gqlClient,
		testUser.ID,
		ent.UpdateUserInput{
			Name: &newName,
		},
		func(t *testing.T, id int, err error) {
			require.NoError(t, err)
			require.NotZero(t, id)
			bobbyd := graph.User.GetX(ctx, id)
			assert.Equal(t, newName, bobbyd.Name)
			assert.Equal(t, "https://upload.wikimedia.org/wikipedia/commons/0/02/Bob_Dylan_-_Azkena_Rock_Festival_2010_2.jpg", bobbyd.PhotoURL)
			assert.NotEmpty(t, bobbyd.SessionToken)
		},
	))
}

func newUpdateUserTest(gqlClient *client.Client, id int, input ent.UpdateUserInput, checks ...func(t *testing.T, id int, err error)) func(t *testing.T) {
	return func(t *testing.T) {
		// Define the mutatation for testing, taking the input as a variable
		mut := `mutation newUpdateUserTest($userID: ID!, $input: UpdateUserInput!) { updateUser(userID: $userID, input:$input) { id } }`

		// Make our request to the GraphQL API
		var resp struct {
			UpdateUser struct{ ID string }
		}
		err := gqlClient.Post(mut, &resp,
			client.Var("userID", id),
			client.Var("input", map[string]interface{}{
				"name":        input.Name,
				"isactivated": input.IsActivated,
				"isadmin":     input.IsAdmin,
				"photourl":    input.PhotoURL,
			}),
		)

		// Run checks with error (if any) and resulting id
		for _, check := range checks {
			check(t, convertID(resp.UpdateUser.ID), err)
		}
	}
}
