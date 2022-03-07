package graphql_test

import (
	"context"
	"testing"

	"github.com/99designs/gqlgen/client"
	"github.com/99designs/gqlgen/graphql/handler"
	"github.com/kcarretto/realm/tavern/ent/enttest"
	"github.com/kcarretto/realm/tavern/ent/tag"
	"github.com/kcarretto/realm/tavern/graphql"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	_ "github.com/mattn/go-sqlite3"
)

// TestCreateTag ensures the createTag mutation exhibits expected behavior.
func TestCreateTag(t *testing.T) {
	// Initialize Test Context
	ctx := context.Background()

	// Initialize DB Backend
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	// Initialize sample data
	existingTarget := graph.Target.Create().
		SetName("target1").
		SetForwardConnectIP("10.0.0.1").
		SaveX(ctx)
	existingTag := graph.Tag.Create().
		SetName("tag1").
		SaveX(ctx)

	// Create a new GraphQL client
	gqlClient := client.New(handler.NewDefaultServer(graphql.NewSchema(graph)))

	// Run tests
	t.Run("New", newCreateTagTest(
		gqlClient,
		graphql.CreateTagInput{
			Name:      "linux",
			TargetIDs: []int{existingTarget.ID},
		},
		func(t *testing.T, id int, err error) {
			require.NoError(t, err)
			assert.NotZero(t, id)

			tag := graph.Tag.GetX(ctx, id)
			assert.Equal(t, "linux", tag.Name)

			targets, err := tag.Targets(ctx)
			require.NoError(t, err)
			require.Len(t, targets, 1)
			assert.Equal(t, existingTarget.ID, targets[0].ID)
		},
	))
	t.Run("Duplicate", newCreateTagTest(
		gqlClient,
		graphql.CreateTagInput{
			Name: existingTag.Name,
		},
		func(t *testing.T, id int, err error) {
			assert.ErrorContains(t, err, "ent: constraint failed: UNIQUE constraint failed: tags.name")
			assert.Zero(t, id)
		},
	))
}

func newCreateTagTest(gqlClient *client.Client, input graphql.CreateTagInput, checks ...func(t *testing.T, id int, err error)) func(t *testing.T) {
	return func(t *testing.T) {
		// Define the mutatation for testing, taking the input as a variable
		mut := `mutation CreateTag($input: CreateTagInput!) { createTag(tag:$input) { id } }`

		// Make our request to the GraphQL API
		var resp struct {
			CreateTag struct{ ID string }
		}
		err := gqlClient.Post(mut, &resp, client.Var("input", input))

		// Run checks with error (if any) and resulting Tag ID
		for _, check := range checks {
			check(t, convertID(resp.CreateTag.ID), err)
		}
	}
}

// TestUpdateTag ensures the updateTag mutation exhibits expected behavior.
func TestUpdateTag(t *testing.T) {
	// Initialize Test Context
	ctx := context.Background()

	// Initialize DB Backend
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	// Initialize sample data
	existingTarget := graph.Target.Create().
		SetName("target1").
		SetForwardConnectIP("10.0.0.1").
		SaveX(ctx)
	existingTarget2 := graph.Target.Create().
		SetName("target2").
		SetForwardConnectIP("10.0.0.2").
		SaveX(ctx)
	existingTag := graph.Tag.Create().
		SetName("tag1").
		AddTargetIDs(existingTarget.ID).
		SaveX(ctx)
	existingTag2 := graph.Tag.Create().
		SetName("tag2").
		SaveX(ctx)

	// Create a new GraphQL client
	gqlClient := client.New(handler.NewDefaultServer(graphql.NewSchema(graph)))

	// Run tests
	t.Run("NoUpdate", newUpdateTagTest(
		gqlClient,
		graphql.UpdateTagInput{
			ID: existingTag.ID,
		},
		func(t *testing.T, id int, err error) {
			require.NoError(t, err)
			assert.NotZero(t, id)
			assert.Equal(t, existingTag.ID, id)

			tag := graph.Tag.GetX(ctx, id)
			assert.Equal(t, existingTag.Name, tag.Name)
		},
	))
	var (
		expectedName string = "new-name"
	)
	t.Run("UpdateName", newUpdateTagTest(
		gqlClient,
		graphql.UpdateTagInput{
			ID:   existingTag.ID,
			Name: &expectedName,
		},
		func(t *testing.T, id int, err error) {
			require.NoError(t, err)
			assert.NotZero(t, id)
			assert.Equal(t, existingTag.ID, id)

			tag := graph.Tag.GetX(ctx, id)
			assert.Equal(t, expectedName, tag.Name)
		},
	))
	t.Run("ChangeTargets", newUpdateTagTest(
		gqlClient,
		graphql.UpdateTagInput{
			ID:              existingTag.ID,
			Name:            &expectedName,
			AddTargetIDs:    []int{existingTarget2.ID},
			RemoveTargetIDs: []int{existingTarget.ID},
		},
		func(t *testing.T, id int, err error) {
			require.NoError(t, err)
			assert.NotZero(t, id)
			assert.Equal(t, existingTag.ID, id)

			tag := graph.Tag.GetX(ctx, id)
			assert.Equal(t, expectedName, tag.Name)

			targets, err := tag.Targets(ctx)
			require.NoError(t, err)
			require.Len(t, targets, 1)
			assert.Equal(t, existingTarget2.ID, targets[0].ID)
		},
	))
	t.Run("NotExists", newUpdateTagTest(
		gqlClient,
		graphql.UpdateTagInput{
			ID: -1,
		},
		func(t *testing.T, id int, err error) {
			assert.Zero(t, id)
			assert.ErrorContains(t, err, "ent: tag not found")
		},
	))
	t.Run("Duplicate", newUpdateTagTest(
		gqlClient,
		graphql.UpdateTagInput{
			ID:   existingTag.ID,
			Name: &existingTag2.Name,
		},
		func(t *testing.T, id int, err error) {
			assert.Zero(t, id)
			assert.ErrorContains(t, err, "ent: constraint failed: UNIQUE constraint failed: tags.name")
		},
	))
}

func newUpdateTagTest(gqlClient *client.Client, input graphql.UpdateTagInput, checks ...func(t *testing.T, id int, err error)) func(t *testing.T) {
	return func(t *testing.T) {
		// Define the mutatation for testing, taking the input as a variable
		mut := `mutation UpdateTag($input: UpdateTagInput!) { updateTag(tag:$input) { id } }`

		// Make our request to the GraphQL API
		var resp struct {
			UpdateTag struct{ ID string }
		}
		err := gqlClient.Post(mut, &resp, client.Var("input", input))

		// Run checks with error (if any) and resulting Tag ID
		for _, check := range checks {
			check(t, convertID(resp.UpdateTag.ID), err)
		}
	}
}

// TestDeleteTag ensures the deleteTag mutation exhibits expected behavior.
func TestDeleteTag(t *testing.T) {
	// Initialize Test Context
	ctx := context.Background()

	// Initialize DB Backend
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	// Initialize sample data
	existingTarget := graph.Target.Create().
		SetName("target1").
		SetForwardConnectIP("10.0.0.1").
		SaveX(ctx)
	existingTag := graph.Tag.Create().
		SetName("delete-me").
		AddTargetIDs(existingTarget.ID).
		SaveX(ctx)
	existingTag2 := graph.Tag.Create().
		SetName("keep-me").
		AddTargetIDs(existingTarget.ID).
		SaveX(ctx)

	// Create a new GraphQL client
	gqlClient := client.New(handler.NewDefaultServer(graphql.NewSchema(graph)))

	// Run tests
	t.Run("Existing", newDeleteTagTest(
		gqlClient,
		existingTag.ID,
		func(t *testing.T, id int, err error) {
			require.NoError(t, err)
			assert.NotZero(t, id)
			assert.Equal(t, existingTag.ID, id)

			exists := graph.Tag.Query().
				Where(tag.ID(id)).
				ExistX(ctx)
			assert.False(t, exists)

			tags, err := graph.Target.GetX(ctx, existingTarget.ID).
				Tags(ctx)
			require.NoError(t, err)
			require.Len(t, tags, 1)
			assert.Equal(t, existingTag2.ID, tags[0].ID)
		},
	))
	t.Run("NotExists", newDeleteTagTest(
		gqlClient,
		-1,
		func(t *testing.T, id int, err error) {
			assert.Zero(t, id)
			assert.ErrorContains(t, err, "ent: tag not found")
		},
	))
}

func newDeleteTagTest(gqlClient *client.Client, id int, checks ...func(t *testing.T, id int, err error)) func(t *testing.T) {
	return func(t *testing.T) {
		// Define the mutatation for testing, taking the input as a variable
		mut := `mutation DeleteTag($input: ID!) { deleteTag(id:$input) }`

		// Make our request to the GraphQL API
		var resp struct {
			DeleteTag string
		}
		err := gqlClient.Post(mut, &resp, client.Var("input", id))

		// Run checks with error (if any) and resulting Tag ID
		for _, check := range checks {
			check(t, convertID(resp.DeleteTag), err)
		}
	}
}
