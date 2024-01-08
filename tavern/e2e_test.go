package main_test

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http/httptest"
	"strconv"
	"testing"

	tavern "github.com/kcarretto/realm/tavern"
	"github.com/kcarretto/realm/tavern/internal/ent"
	"github.com/kcarretto/realm/tavern/internal/ent/beacon"
	"github.com/kcarretto/realm/tavern/internal/ent/enttest"
	"github.com/kcarretto/realm/tavern/internal/ent/host"
	"github.com/kcarretto/realm/tavern/internal/ent/quest"
	"github.com/kcarretto/realm/tavern/internal/ent/tome"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestEndToEnd(t *testing.T) {
	// Setup
	ctx := context.Background()
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()
	srv, err := tavern.NewServer(ctx, tavern.ConfigureMySQLFromClient(graph))
	require.NoError(t, err)
	defer srv.Close()
	ts := httptest.NewUnstartedServer(srv.HTTP.Handler)
	ts.Config = srv.HTTP
	ts.Start()
	defer ts.Close()

	// Status Test
	t.Run("StatusHandler", func(t *testing.T) {
		resp, err := ts.Client().Get(testURL(ts, "status"))
		require.NoError(t, err)
		body, err := io.ReadAll(resp.Body)
		require.NoError(t, err)
		assert.Equal(t, tavern.OKStatusText, string(body))
	})

	// Helper for performing beacon callbacks
	beaconCallback := func(t *testing.T, identifier string) (*ent.Beacon, claimTasksResponse) {
		req := graphQLQuery{
			OperationName: "ClaimTasks",
			Query: `mutation ClaimTasks($input: ClaimTasksInput!) {
				claimTasks(input: $input) {
					id
					quest {
						id
						tome {
							id
						}
						bundle {
							id
						}
			  		}
				}
			}`,
			Variables: map[string]any{
				"input": map[string]any{
					"principal":        "root",
					"hostname":         "some_hostname",
					"hostPlatform":     host.PlatformUnknown,
					"beaconIdentifier": identifier,
					"hostIdentifier":   "uniquely_identifies_host.For_example_a_serial_number",
					"agentIdentifier":  "uniquely_identifies_this_agent",
				},
			},
		}
		data, err := json.Marshal(req)
		require.NoError(t, err)
		resp, err := ts.Client().Post(testURL(ts, "graphql"), "application/json", bytes.NewBuffer(data))
		require.NoError(t, err)
		body, err := io.ReadAll(resp.Body)
		require.NoError(t, err)

		var graphQLResp claimTasksResponse
		err = json.Unmarshal(body, &graphQLResp)
		require.NoError(t, err)

		testBeacon, err := graph.Beacon.Query().
			Where(beacon.Identifier(identifier)).
			Only(ctx)
		require.NoError(t, err)
		assert.NotZero(t, testBeacon.LastSeenAt)
		return testBeacon, graphQLResp
	}

	var (
		createdQuestID  int
		createdTaskID   int
		defaultTomeID   int
		createdBundleID int
	)

	// Beacon First Callback
	firstBeaconIdentifier := "first_beacon"
	t.Run("BeaconFirstCallback", func(t *testing.T) {
		_, resp := beaconCallback(t, firstBeaconIdentifier)
		assert.Len(t, resp.Data.ClaimTasks, 0)
	})

	// Create a Quest (assumes a default tome has been properly configured)
	t.Run("CreateQuest", func(t *testing.T) {
		testBeacon, err := graph.Beacon.Query().
			Where(beacon.Identifier(firstBeaconIdentifier)).
			Only(ctx)
		require.NoError(t, err)

		testTome, err := graph.Tome.Query().Where(tome.NameEQ("example")).First(ctx)
		require.NoError(t, err)
		require.NotNil(t, testTome)
		defaultTomeID = testTome.ID

		req := graphQLQuery{
			OperationName: "CreateQuest",
			Query:         "mutation CreateQuest($beaconIDs: [ID!]!, $input: CreateQuestInput!) { createQuest(beaconIDs: $beaconIDs, input: $input) { id } }",
			Variables: map[string]any{
				"beaconIDs": []int{testBeacon.ID},
				"input": map[string]any{
					"name":   "unit test",
					"tomeID": testTome.ID,
				},
			},
		}
		data, err := json.Marshal(req)
		require.NoError(t, err)
		_, err = ts.Client().Post(testURL(ts, "graphql"), "application/json", bytes.NewBuffer(data))
		require.NoError(t, err)

		testQuest, err := graph.Quest.Query().
			Where(quest.Name("unit test")).
			Only(ctx)
		require.NoError(t, err)
		createdQuestID = testQuest.ID

		testBundle, err := testQuest.Bundle(ctx)
		require.NoError(t, err)
		require.NotNil(t, testBundle)
		createdBundleID = testBundle.ID

		testTasks, err := testQuest.Tasks(ctx)
		require.NoError(t, err)
		require.NotEmpty(t, testTasks)
		assert.Len(t, testTasks, 1)
		createdTaskID = testTasks[0].ID

		testTaskBeacon, err := testTasks[0].Beacon(ctx)
		require.NoError(t, err)
		assert.Equal(t, testBeacon.ID, testTaskBeacon.ID)
	})

	// Beacon Second Callback
	t.Run("BeaconSecondCallback", func(t *testing.T) {
		_, resp := beaconCallback(t, firstBeaconIdentifier)
		require.NotEmpty(t, resp.Data.ClaimTasks)
		assert.Len(t, resp.Data.ClaimTasks, 1)

		questID := convertID(resp.Data.ClaimTasks[0].Quest.ID)
		assert.Equal(t, createdQuestID, questID)

		taskID := convertID(resp.Data.ClaimTasks[0].ID)
		assert.Equal(t, createdTaskID, taskID)

		tomeID := convertID(resp.Data.ClaimTasks[0].Quest.Tome.ID)
		assert.Equal(t, defaultTomeID, tomeID)

		bundleID := convertID(resp.Data.ClaimTasks[0].Quest.Bundle.ID)
		assert.Equal(t, createdBundleID, bundleID)
	})

}

type graphQLQuery struct {
	OperationName string         `json:"operationName"`
	Query         string         `json:"query"`
	Variables     map[string]any `json:"variables"`
}

type claimTasksResponse struct {
	Data struct {
		ClaimTasks []struct {
			ID    string `json:"id"`
			Quest struct {
				ID   string `json:"id"`
				Tome struct {
					ID string `json:"id"`
				} `json:"tome"`
				Bundle struct {
					ID string `json:"id"`
				} `json:"bundle"`
			} `json:"quest"`
		} `json:"claimTasks"`
	} `json:"data"`
}

func testURL(ts *httptest.Server, endpoint string) string {
	return fmt.Sprintf("%s/%s", ts.URL, endpoint)
}

// convertID turns a GraphQL string ID response into an int.
func convertID(id string) int {
	if id == "" {
		return 0
	}
	intID, err := strconv.Atoi(id)
	if err != nil {
		panic(err)
	}
	return intID
}
