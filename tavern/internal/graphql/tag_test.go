package graphql_test

import (
	"context"
	"testing"
	"time"

	"github.com/99designs/gqlgen/client"
	"github.com/99designs/gqlgen/graphql/handler"
	"github.com/kcarretto/realm/tavern/internal/auth"
	"github.com/kcarretto/realm/tavern/internal/ent"
	"github.com/kcarretto/realm/tavern/internal/ent/enttest"
	"github.com/kcarretto/realm/tavern/internal/ent/tag"
	"github.com/kcarretto/realm/tavern/internal/graphql"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestTagMutations(t *testing.T) {
	// Setup
	ctx := context.Background()
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()
	srv := auth.AuthDisabledMiddleware(handler.NewDefaultServer(graphql.NewSchema(graph)), graph)
	gqlClient := client.New(srv)

	// Initialize sample data
	testTags := []*ent.Tag{
		graph.Tag.Create().
			SetKind(tag.KindGroup).
			SetName("TestTag1").
			SaveX(ctx),
	}
	testBeacons := []*ent.Beacon{
		graph.Beacon.Create().
			SetPrincipal("admin").
			SetAgentIdentifier("TEST").
			SetIdentifier("SOME_ID").
			SetHostIdentifier("SOME_HOST_ID").
			SetHostname("SOME_HOSTNAME").
			SetLastSeenAt(time.Now().Add(-10 * time.Minute)).
			SaveX(ctx),
		graph.Beacon.Create().
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

		t.Run("WithoutBeacons", func(t *testing.T) {
			expected := map[string]any{
				"name": "TestTagWithoutBeacons",
				"kind": tag.KindGroup,
			}
			id, err := createTag(expected)
			require.NoError(t, err)
			require.NotZero(t, id)
			testTag := graph.Tag.GetX(ctx, id)
			assert.Equal(t, expected["name"], testTag.Name)
			assert.Equal(t, expected["kind"], testTag.Kind)
		})
		t.Run("WithBeacons", func(t *testing.T) {
			expectedBeaconIDs := []int{testBeacons[0].ID}
			expected := map[string]any{
				"name":      "TestTagWithBeacons",
				"kind":      tag.KindGroup,
				"beaconIDs": expectedBeaconIDs,
			}
			id, err := createTag(expected)
			require.NoError(t, err)
			require.NotZero(t, id)
			testTag := graph.Tag.GetX(ctx, id)
			assert.Equal(t, expected["name"], testTag.Name)
			assert.Equal(t, expected["kind"], testTag.Kind)
			testTagBeacons, err := testTag.Beacons(ctx)
			require.NoError(t, err)
			assert.Len(t, testTagBeacons, 1)
			assert.Equal(t, expectedBeaconIDs[0], testTagBeacons[0].ID)
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

		t.Run("ModifyBeacons", func(t *testing.T) {
			expectedBeaconIDs := []int{testBeacons[0].ID}
			expected := map[string]any{
				"addBeaconIDs":    expectedBeaconIDs,
				"removeBeaconIDs": testBeacons[1].ID,
			}
			id, err := updateTag(testTags[0].ID, expected)
			require.NoError(t, err)
			assert.NotZero(t, id)
			testTag := graph.Tag.GetX(ctx, id)
			testTagBeacons, err := testTag.Beacons(ctx)
			require.NoError(t, err)
			assert.Len(t, testTagBeacons, 1)
			assert.Equal(t, expectedBeaconIDs[0], testTagBeacons[0].ID)
		})
	})
}
