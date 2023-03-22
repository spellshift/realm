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
	"github.com/kcarretto/realm/tavern/ent"
	"github.com/kcarretto/realm/tavern/ent/enttest"
	"github.com/kcarretto/realm/tavern/ent/job"
	"github.com/kcarretto/realm/tavern/ent/session"
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

	// Helper for performing session callbacks
	sessionCallback := func(t *testing.T, identifier string) (*ent.Session, claimTasksResponse) {
		req := graphQLQuery{
			OperationName: "ClaimTasks",
			Query: `mutation ClaimTasks($input: ClaimTasksInput!) {
				claimTasks(input: $input) {
					id
					job {
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
					"principal":         "root",
					"hostname":          "some_hostname",
					"hostPlatform":      session.HostPlatformUnknown,
					"sessionIdentifier": identifier,
					"hostIdentifier":    "uniquely_identifies_host.For_example_a_serial_number",
					"agentIdentifier":   "uniquely_identifies_this_agent",
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

		testSession, err := graph.Session.Query().
			Where(session.Identifier(identifier)).
			Only(ctx)
		require.NoError(t, err)
		assert.NotZero(t, testSession.LastSeenAt)
		return testSession, graphQLResp
	}

	var (
		createdJobID    int
		createdTaskID   int
		defaultTomeID   int
		createdBundleID int
	)

	// Session First Callback
	firstSessionIdentifier := "first_session"
	t.Run("SessionFirstCallback", func(t *testing.T) {
		_, resp := sessionCallback(t, firstSessionIdentifier)
		assert.Len(t, resp.Data.ClaimTasks, 0)
	})

	// Create a Job (assumes a default tome has been properly configured)
	t.Run("CreateJob", func(t *testing.T) {
		testSession, err := graph.Session.Query().
			Where(session.Identifier(firstSessionIdentifier)).
			Only(ctx)
		require.NoError(t, err)

		testTome, err := graph.Tome.Query().First(ctx)
		require.NoError(t, err)
		require.NotNil(t, testTome)
		defaultTomeID = testTome.ID

		req := graphQLQuery{
			OperationName: "CreateJob",
			Query:         "mutation CreateJob($sessionIDs: [ID!]!, $input: CreateJobInput!) { createJob(sessionIDs: $sessionIDs, input: $input) { id } }",
			Variables: map[string]any{
				"sessionIDs": []int{testSession.ID},
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

		testJob, err := graph.Job.Query().
			Where(job.Name("unit test")).
			Only(ctx)
		require.NoError(t, err)
		createdJobID = testJob.ID

		testBundle, err := testJob.Bundle(ctx)
		require.NoError(t, err)
		require.NotNil(t, testBundle)
		createdBundleID = testBundle.ID

		testTasks, err := testJob.Tasks(ctx)
		require.NoError(t, err)
		require.NotEmpty(t, testTasks)
		assert.Len(t, testTasks, 1)
		createdTaskID = testTasks[0].ID

		testTaskSession, err := testTasks[0].Session(ctx)
		require.NoError(t, err)
		assert.Equal(t, testSession.ID, testTaskSession.ID)
	})

	// Session Second Callback
	t.Run("SessionSecondCallback", func(t *testing.T) {
		_, resp := sessionCallback(t, firstSessionIdentifier)
		require.NotEmpty(t, resp.Data.ClaimTasks)
		assert.Len(t, resp.Data.ClaimTasks, 1)

		jobID := convertID(resp.Data.ClaimTasks[0].Job.ID)
		assert.Equal(t, createdJobID, jobID)

		taskID := convertID(resp.Data.ClaimTasks[0].ID)
		assert.Equal(t, createdTaskID, taskID)

		tomeID := convertID(resp.Data.ClaimTasks[0].Job.Tome.ID)
		assert.Equal(t, defaultTomeID, tomeID)

		bundleID := convertID(resp.Data.ClaimTasks[0].Job.Bundle.ID)
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
			ID  string `json:"id"`
			Job struct {
				ID   string `json:"id"`
				Tome struct {
					ID string `json:"id"`
				} `json:"tome"`
				Bundle struct {
					ID string `json:"id"`
				} `json:"bundle"`
			} `json:"job"`
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
