package graphql_test

import (
	"context"
	"testing"

	"github.com/99designs/gqlgen/client"
	"github.com/99designs/gqlgen/graphql/handler"
	"github.com/kcarretto/realm/tavern/internal/auth"
	"github.com/kcarretto/realm/tavern/internal/ent"
	"github.com/kcarretto/realm/tavern/internal/ent/enttest"
	"github.com/kcarretto/realm/tavern/internal/graphql"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestUsersQuery ensures that the users query works as expected.
func TestUsersQuery(t *testing.T) {
	// Setup
	ctx := context.Background()
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()
	srv := auth.AuthDisabledMiddleware(handler.NewDefaultServer(graphql.NewSchema(graph)), graph)
	gqlClient := client.New(srv)

	// Initialize sample data
	testUser := graph.User.Create().
		SetName("bobdylan").
		SetIsActivated(false).
		SetIsAdmin(true).
		SetOauthID("likearollingstone").
		SetPhotoURL("https://upload.wikimedia.org/wikipedia/commons/0/02/Bob_Dylan_-_Azkena_Rock_Festival_2010_2.jpg").
		SaveX(ctx)

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

func TestQuestsQuery(t *testing.T) {
	// Setup
	ctx := context.Background()
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()
	srv := auth.AuthDisabledMiddleware(handler.NewDefaultServer(graphql.NewSchema(graph)), graph)
	gqlClient := client.New(srv)

	// Initialize sample data
	tome := graph.Tome.Create().
		SetName("testTome").
		SetDescription("For testing!").
		SetEldritch("Testing!").
		SaveX(ctx)
	testQuests := []*ent.Quest{
		graph.Quest.Create().
			SetName("Test 1").
			SetTome(tome).
			SaveX(ctx),
		graph.Quest.Create().
			SetName("Test 2").
			SetTome(tome).
			SaveX(ctx),
	}

	t.Run("All", func(t *testing.T) {
		query := `
query AllQuests($where: QuestWhereInput) {
	quests(where: $where) {
		id
	}
}`
		queryQuests := func(where map[string]any) ([]int, error) {
			var resp struct {
				Quests []struct{ ID string } `json:"quests"`
			}
			if where == nil {
				where = make(map[string]any)
			}
			err := gqlClient.Post(query, &resp, client.Var("where", where))
			if err != nil {
				return nil, err
			}

			ids := make([]int, 0, len(resp.Quests))
			for _, j := range resp.Quests {
				ids = append(ids, convertID(j.ID))
			}
			return ids, nil
		}

		ids, err := queryQuests(nil)
		require.NoError(t, err)
		assert.Contains(t, ids, testQuests[0].ID)
		assert.Contains(t, ids, testQuests[1].ID)
	})
}
