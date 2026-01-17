package graphql_test

import (
	"context"
	"testing"

	"github.com/99designs/gqlgen/client"
	"github.com/99designs/gqlgen/graphql/handler"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/graphql"
	tavernhttp "realm.pub/tavern/internal/http"
	"realm.pub/tavern/tomes"
)

func TestTomeMutations(t *testing.T) {
	// Setup
	ctx := context.Background()
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	// Initialize Git Importer
	git := tomes.NewGitImporter(graph)

	srv := tavernhttp.NewServer(
		tavernhttp.RouteMap{
			"/graphql": handler.NewDefaultServer(graphql.NewSchema(graph, git)),
		},
		tavernhttp.WithAuthenticationBypass(graph),
	)
	gqlClient := client.New(srv, client.Path("/graphql"))

	// Initialize sample data
	testAssets := []*ent.Asset{
		graph.Asset.Create().
			SetName("TestAsset1").
			SetContent([]byte("supersecretfile")).
			SaveX(ctx),
		graph.Asset.Create().
			SetName("TestAsset2").
			SetContent([]byte("expect_this")).
			SaveX(ctx),
	}

	t.Run("CreateTome", func(t *testing.T) {
		// Define the CreateTome mutation
		mut := `
mutation newCreateTomeTest($input: CreateTomeInput!) {
	createTome(input: $input) {
		id
	}
}`
		// Create a closure to execute the mutation and return the id(s)
		createTome := func(input map[string]any) (int, error) {
			var resp struct {
				CreateTome struct{ ID string }
			}
			err := gqlClient.Post(mut, &resp, client.Var("input", input))
			if err != nil {
				return 0, err
			}
			return convertID(resp.CreateTome.ID), nil
		}

		/*
		* Test when the `createTome` mutation is run without files.
		*
		* Expected that the Tome is created and it's ID is returned.
		 */
		t.Run("WithoutFiles", func(t *testing.T) {
			expected := map[string]any{
				"name":        "TestTome",
				"description": "Helps us make sure this all works",
				"author":      "kcarretto",
				"eldritch":    `print("hello world")`,
			}
			id, err := createTome(expected)
			require.NoError(t, err)
			require.NotZero(t, id)
			testTome := graph.Tome.GetX(ctx, id)
			assert.Equal(t, expected["name"], testTome.Name)
			assert.Equal(t, expected["description"], testTome.Description)
			assert.Equal(t, expected["eldritch"], testTome.Eldritch)
			assert.NotZero(t, testTome.Hash)
			assert.NotZero(t, testTome.CreatedAt)
			assert.NotZero(t, testTome.LastModifiedAt)
		})

		/*
		* Test when the `createTome` mutation is run with assets.
		*
		* Expected that the Tome is created and it's ID is returned.
		 */
		t.Run("WithAssets", func(t *testing.T) {
			expectedAssetIDs := []int{testAssets[0].ID, testAssets[1].ID}
			expected := map[string]any{
				"name":        "TestTomeWithAssets",
				"description": "Helps us make sure this all works",
				"author":      "kcarretto",
				"eldritch":    `print("hello world")`,
				"assetIDs":    expectedAssetIDs,
			}
			id, err := createTome(expected)
			require.NoError(t, err)
			require.NotZero(t, id)
			testTome := graph.Tome.GetX(ctx, id)
			assert.Equal(t, expected["name"], testTome.Name)
			assert.Equal(t, expected["description"], testTome.Description)
			assert.Equal(t, expected["eldritch"], testTome.Eldritch)
			assert.NotZero(t, testTome.Hash)
			assert.NotZero(t, testTome.CreatedAt)
			assert.NotZero(t, testTome.LastModifiedAt)
			testTomeAssets, err := testTome.QueryAssets().All(ctx)
			require.NoError(t, err)
			assert.Len(t, testTomeAssets, 2)
			assert.Equal(t, expectedAssetIDs[0], testTomeAssets[0].ID)
			assert.Equal(t, expectedAssetIDs[1], testTomeAssets[1].ID)
		})

	})

}
