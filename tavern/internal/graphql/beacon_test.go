package graphql_test

import (
	"context"
	"fmt"
	"testing"
	"time"

	"github.com/99designs/gqlgen/client"
	"github.com/99designs/gqlgen/graphql/handler"
	"github.com/kcarretto/realm/tavern/internal/auth"
	"github.com/kcarretto/realm/tavern/internal/ent"
	"github.com/kcarretto/realm/tavern/internal/ent/beacon"
	"github.com/kcarretto/realm/tavern/internal/ent/enttest"
	"github.com/kcarretto/realm/tavern/internal/ent/host"
	"github.com/kcarretto/realm/tavern/internal/ent/tag"
	"github.com/kcarretto/realm/tavern/internal/graphql"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestBeaconMutations(t *testing.T) {
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
		graph.Tag.Create().
			SetKind(tag.KindGroup).
			SetName("TestTag2").
			SaveX(ctx),
	}
	testHost := graph.Host.Create().
		SetIdentifier("SOME_HOST_ID").
		SetName("SOME_HOSTNAME").
		AddTags(testTags[1]).
		SaveX(ctx)

	testBeacons := []*ent.Beacon{
		graph.Beacon.Create().
			SetPrincipal("admin").
			SetAgentIdentifier("TEST").
			SetIdentifier("SOME_ID").
			SetHost(testHost).
			SetLastSeenAt(time.Now().Add(-10 * time.Minute)).
			SaveX(ctx),
		graph.Beacon.Create().
			SetIdentifier("ANOTHER_ID").
			SetHost(testHost).
			SetLastSeenAt(time.Now().Add(-10 * time.Minute)).
			SaveX(ctx),
	}
	testTome := graph.Tome.Create().
		SetName("Test Tome").
		SetDescription("Ensures the world feels greeted").
		SetEldritch(`print("Hello World!")`).
		SaveX(ctx)
	testQuest := graph.Quest.Create().
		SetName("howdy-ho").
		SetTome(testTome).
		SaveX(ctx)
	testTasks := []*ent.Task{
		graph.Task.Create().
			SetQuest(testQuest).
			SetBeacon(testBeacons[0]).
			SaveX(ctx),
	}

	t.Run("ClaimTasks", func(t *testing.T) {
		// Define the ClaimTasks mutation
		mut := `
mutation newClaimTasksTest($input: ClaimTasksInput!) {
	claimTasks(input: $input) {
		id
		quest {
			id
		}
	}
}`
		// Create a closure to execute the mutation
		claimTasks := func(input map[string]any) ([]int, error) {
			// Make our request to the GraphQL API
			var resp struct {
				ClaimTasks []struct {
					ID    string
					Quest struct {
						ID string
					} `json:"quest"`
				}
			}
			err := gqlClient.Post(mut, &resp, client.Var("input", input))
			if err != nil {
				return nil, err
			}

			// Parse Task IDs from response
			ids := make([]int, 0, len(resp.ClaimTasks))
			for _, task := range resp.ClaimTasks {
				ids = append(ids, convertID(task.ID))
			}
			return ids, nil
		}

		/*
		* Test when the `claimTasks` mutation is run by a beacon that does not exist.
		*
		* Expected that the beacon is created, but no tasks are returned.
		 */
		t.Run("NewBeacon", func(t *testing.T) {
			expectedIdentifier := "NEW_ID"
			expected := map[string]any{
				"principal":        "newuser",
				"hostname":         "NEW_HOSTNAME",
				"hostPlatform":     host.PlatformWindows,
				"beaconIdentifier": expectedIdentifier,
				"hostIdentifier":   "NEW_HOST_ID",
				"agentIdentifier":  "NEW_AGENT_ID",
			}
			ids, err := claimTasks(expected)
			require.NoError(t, err)
			assert.Empty(t, ids)
			testBeacon, err := graph.Beacon.Query().
				Where(beacon.Identifier(expectedIdentifier)).
				Only(ctx)
			require.NoError(t, err)
			assert.NotNil(t, testBeacon)
			assert.NotZero(t, testBeacon.LastSeenAt)
			assert.Equal(t, expected["principal"], testBeacon.Principal)
			assert.Equal(t, expected["beaconIdentifier"], testBeacon.Identifier)
			assert.Equal(t, expected["agentIdentifier"], testBeacon.AgentIdentifier)

			testHost, err := testBeacon.Host(ctx)
			require.NoError(t, err)
			require.NotNil(t, testHost)
			assert.Equal(t, expected["hostIdentifier"], testHost.Identifier)
			assert.Equal(t, expected["hostname"], testHost.Name)

		})

		/*
		 * Test when the `claimTasks` mutation is run by a beacon that already exists.
		 *
		 * Expected that the beacon is updated, and our test task is returned.
		 */
		t.Run("ExistingBeacon", func(t *testing.T) {
			expected := map[string]any{
				"principal":        "admin",
				"hostname":         "SOME_HOSTNAME",
				"hostPlatform":     host.PlatformMacOS,
				"hostPrimaryIP":    "10.0.0.1",
				"beaconIdentifier": "SOME_ID",
				"hostIdentifier":   "SOME_HOST_ID",
				"agentIdentifier":  "SOME_AGENT_ID",
			}
			ids, err := claimTasks(expected)
			require.NoError(t, err)
			assert.Len(t, ids, 1)
			assert.Equal(t, testTasks[0].ID, ids[0])
			testTask := graph.Task.GetX(ctx, testTasks[0].ID)
			assert.NotZero(t, testTask.ClaimedAt)

			testBeacon, err := testTask.Beacon(ctx)
			require.NoError(t, err)
			assert.Equal(t, testBeacons[0].ID, testBeacon.ID)
			assert.NotZero(t, testBeacon.LastSeenAt)
			assert.Equal(t, expected["principal"], testBeacon.Principal)
			assert.Equal(t, expected["beaconIdentifier"], testBeacon.Identifier)
			assert.Equal(t, expected["agentIdentifier"], testBeacon.AgentIdentifier)

			testHost, err := testBeacon.Host(ctx)
			require.NoError(t, err)
			require.NotNil(t, testHost)
			assert.Equal(t, expected["hostIdentifier"], testHost.Identifier)
			assert.Equal(t, expected["hostname"], testHost.Name)
			assert.Equal(t, expected["hostPlatform"], testHost.Platform)
			assert.Equal(t, expected["hostPrimaryIP"], testHost.PrimaryIP)
		})
	})

	t.Run("SubmitTaskResult", func(t *testing.T) {
		// Define the SubmitTaskResult mutation
		mut := `
mutation newSubmitTaskResultTest($input: SubmitTaskResultInput!) {
	submitTaskResult(input: $input) {
		id
	}
}`
		// Create a closure to execute the mutation
		submitTaskResult := func(input map[string]any) (int, error) {
			// Make our request to the GraphQL API
			var resp struct {
				SubmitTaskResult struct{ ID string }
			}
			err := gqlClient.Post(mut, &resp, client.Var("input", input))
			if err != nil {
				return 0, err
			}

			// Parse Task ID from response
			return convertID(resp.SubmitTaskResult.ID), nil
		}

		expectedExecStartedAt := time.Now()
		t.Run("FirstSubmit", func(t *testing.T) {
			expected := map[string]any{
				"taskID":        fmt.Sprintf("%d", testTasks[0].ID),
				"execStartedAt": expectedExecStartedAt,
				"output":        "one",
			}
			id, err := submitTaskResult(expected)
			require.NoError(t, err)
			assert.NotZero(t, id)
			testTask := graph.Task.GetX(ctx, id)
			assert.Equal(t, testTasks[0].ID, testTask.ID)
			assert.Equal(t, expectedExecStartedAt.UnixNano(), testTask.ExecStartedAt.UnixNano())
			assert.Equal(t, "one", testTask.Output)
			assert.Zero(t, testTask.ExecFinishedAt)
			assert.Zero(t, testTask.Error)
		})
		t.Run("SecondSubmit", func(t *testing.T) {
			expected := map[string]any{
				"taskID":        fmt.Sprintf("%d", testTasks[0].ID),
				"execStartedAt": expectedExecStartedAt,
				"output":        "_two",
			}
			id, err := submitTaskResult(expected)
			require.NoError(t, err)
			assert.NotZero(t, id)
			testTask := graph.Task.GetX(ctx, id)
			assert.Equal(t, testTasks[0].ID, testTask.ID)
			assert.Equal(t, expectedExecStartedAt.UnixNano(), testTask.ExecStartedAt.UnixNano())
			assert.Equal(t, "one_two", testTask.Output)
			assert.Zero(t, testTask.ExecFinishedAt)
			assert.Zero(t, testTask.Error)
		})

		expectedExecFinishedAt := time.Now().Add(1 * time.Second)
		t.Run("Complete", func(t *testing.T) {
			expected := map[string]any{
				"taskID":         fmt.Sprintf("%d", testTasks[0].ID),
				"execStartedAt":  expectedExecStartedAt,
				"execFinishedAt": expectedExecFinishedAt,
				"output":         "_three",
				"error":          "uh oh",
			}
			id, err := submitTaskResult(expected)
			require.NoError(t, err)
			assert.NotZero(t, id)
			testTask := graph.Task.GetX(ctx, id)
			assert.Equal(t, testTasks[0].ID, testTask.ID)
			assert.Equal(t, expectedExecStartedAt.UnixNano(), testTask.ExecStartedAt.UnixNano())
			assert.Equal(t, "one_two_three", testTask.Output)
			assert.Equal(t, expectedExecFinishedAt.UnixNano(), testTask.ExecFinishedAt.UnixNano())
			assert.Equal(t, expected["error"], testTask.Error)
		})
	})
}
