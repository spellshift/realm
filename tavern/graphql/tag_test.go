package graphql_test

import (
	"context"
	"testing"
	"time"

	"github.com/99designs/gqlgen/client"
	"github.com/99designs/gqlgen/graphql/handler"
	"github.com/kcarretto/realm/tavern/auth/authtest"
	"github.com/kcarretto/realm/tavern/ent"
	"github.com/kcarretto/realm/tavern/ent/enttest"
	"github.com/kcarretto/realm/tavern/ent/tag"
	"github.com/kcarretto/realm/tavern/graphql"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestTagMutations(t *testing.T) {
	// Setup
	ctx := context.Background()
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()
	srv := authtest.Middleware(handler.NewDefaultServer(graphql.NewSchema(graph)))
	gqlClient := client.New(srv)

	// Initialize sample data
	testTags := []*ent.Tag{
		graph.Tag.Create().
			SetKind(tag.KindGroup).
			SetName("TestTag1").
			SaveX(ctx),
	}
	testSessions := []*ent.Session{
		graph.Session.Create().
			SetPrincipal("admin").
			SetAgentIdentifier("TEST").
			SetIdentifier("SOME_ID").
			SetHostIdentifier("SOME_HOST_ID").
			SetHostname("SOME_HOSTNAME").
			SetLastSeenAt(time.Now().Add(-10 * time.Minute)).
			SaveX(ctx),
		graph.Session.Create().
			SetIdentifier("ANOTHER_ID").
			SetLastSeenAt(time.Now().Add(-10 * time.Minute)).
			AddTags(testTags[0]).
			SaveX(ctx),
	}

	t.Run("CreateTag", func(t *testing.T) {
		// Define the CreateTag mutation
		mut := `
mutation newCreateTagTest($input: CreateTagInput!) {
	createTag(input: $input) {
		id
	}
}`
		createTag := func(input map[string]any) (int, error) {
			var resp struct {
				CreateTag struct{ ID string }
			}
			err := gqlClient.Post(mut, &resp, client.Var("input", input))
			if err != nil {
				return 0, err
			}
			return convertID(resp.CreateTag.ID), nil
		}

		t.Run("WithoutSessions", func(t *testing.T) {
			expected := map[string]any{
				"name": "TestTagWithoutSessions",
				"kind": tag.KindGroup,
			}
			id, err := createTag(expected)
			require.NoError(t, err)
			require.NotZero(t, id)
			testTag := graph.Tag.GetX(ctx, id)
			assert.Equal(t, expected["name"], testTag.Name)
			assert.Equal(t, expected["kind"], testTag.Kind)
		})
		t.Run("WithSessions", func(t *testing.T) {
			expectedSessionIDs := []int{testSessions[0].ID}
			expected := map[string]any{
				"name":       "TestTagWithSessions",
				"kind":       tag.KindGroup,
				"sessionIDs": expectedSessionIDs,
			}
			id, err := createTag(expected)
			require.NoError(t, err)
			require.NotZero(t, id)
			testTag := graph.Tag.GetX(ctx, id)
			assert.Equal(t, expected["name"], testTag.Name)
			assert.Equal(t, expected["kind"], testTag.Kind)
			testTagSessions, err := testTag.Sessions(ctx)
			require.NoError(t, err)
			assert.Len(t, testTagSessions, 1)
			assert.Equal(t, expectedSessionIDs[0], testTagSessions[0].ID)
		})

	})

	t.Run("UpdateTag", func(t *testing.T) {
		// Define the UpdateTag mutation
		mut := `
mutation newUpdateTagTest($tagID: ID!, $input: UpdateTagInput!) {
	updateTag(tagID: $tagID, input: $input) {
		id
	}
}`
		updateTag := func(tagID int, input map[string]any) (int, error) {
			var resp struct {
				UpdateTag struct{ ID string }
			}
			err := gqlClient.Post(
				mut,
				&resp,
				client.Var("tagID", tagID),
				client.Var("input", input),
			)
			if err != nil {
				return 0, err
			}
			return convertID(resp.UpdateTag.ID), nil
		}

		t.Run("ModifySessions", func(t *testing.T) {
			expectedSessionIDs := []int{testSessions[0].ID}
			expected := map[string]any{
				"addSessionIDs":    expectedSessionIDs,
				"removeSessionIDs": testSessions[1].ID,
			}
			id, err := updateTag(testTags[0].ID, expected)
			require.NoError(t, err)
			assert.NotZero(t, id)
			testTag := graph.Tag.GetX(ctx, id)
			testTagSessions, err := testTag.Sessions(ctx)
			require.NoError(t, err)
			assert.Len(t, testTagSessions, 1)
			assert.Equal(t, expectedSessionIDs[0], testTagSessions[0].ID)
		})
	})
}
