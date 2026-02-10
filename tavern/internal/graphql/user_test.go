package graphql_test

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/graphql"
	tavernhttp "realm.pub/tavern/internal/http"
	"realm.pub/tavern/tomes"

	"github.com/99designs/gqlgen/client"
	"github.com/99designs/gqlgen/graphql/handler"
	_ "github.com/mattn/go-sqlite3"
)

// TestUserMutations ensures the user mutations exhibits expected behavior.
func TestUserMutations(t *testing.T) {
	// Setup
	ctx := context.Background()
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	// Initialize Git Importer
	git := tomes.NewGitImporter(graph)

	srv := tavernhttp.NewServer(
		tavernhttp.RouteMap{
			"/graphql": handler.NewDefaultServer(graphql.NewSchema(graph, git, nil, nil)),
		},
		tavernhttp.WithAuthenticationBypass(graph),
	)
	gqlClient := client.New(srv, client.Path("/graphql"))

	// Initialize sample data
	testUser := graph.User.Create().
		SetName("bobdylan").
		SetIsActivated(true).
		SetIsAdmin(true).
		SetOauthID("likearollingstone").
		SetPhotoURL("https://upload.wikimedia.org/wikipedia/commons/0/02/Bob_Dylan_-_Azkena_Rock_Festival_2010_2.jpg").
		SaveX(ctx)

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
				"isActivated": input.IsActivated,
				"isAdmin":     input.IsAdmin,
				"photoURL":    input.PhotoURL,
			}),
		)

		// Run checks with error (if any) and resulting id
		for _, check := range checks {
			check(t, convertID(resp.UpdateUser.ID), err)
		}
	}
}
