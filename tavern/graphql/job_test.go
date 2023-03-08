package graphql_test

import (
	"archive/tar"
	"bytes"
	"compress/gzip"
	"context"
	"io"
	"testing"
	"time"

	"github.com/99designs/gqlgen/client"
	"github.com/99designs/gqlgen/graphql/handler"
	"github.com/kcarretto/realm/tavern/auth"
	"github.com/kcarretto/realm/tavern/ent"
	"github.com/kcarretto/realm/tavern/ent/enttest"
	"github.com/kcarretto/realm/tavern/graphql"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestCreateJob ensures the createJob mutation functions as expected
func TestCreateJob(t *testing.T) {
	// Setup
	ctx := context.Background()
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()
	srv := auth.AuthDisabledMiddleware(handler.NewDefaultServer(graphql.NewSchema(graph)), graph)
	gqlClient := client.New(srv)

	// Initialize sample data
	testSessions := []*ent.Session{
		graph.Session.Create().
			SaveX(ctx),
		graph.Session.Create().
			SaveX(ctx),
	}
	testTome := graph.Tome.Create().
		SetName("Test Tome").
		SetDescription("Ensures the world feels greeted").
		SetEldritch(`print("Hello World!")`).
		SaveX(ctx)

	testFiles := []*ent.File{
		graph.File.Create().
			SetName("TestFile1").
			SetContent([]byte("supersecretfile")).
			SaveX(ctx),
		graph.File.Create().
			SetName("TestFile2").
			SetContent([]byte("expect_this")).
			SaveX(ctx),
	}
	testTomeWithFiles := graph.Tome.Create().
		SetName("Test Tome With Files").
		SetDescription("Ensures the world feels greeted").
		SetEldritch(`print("Hello World!")`).
		AddFiles(testFiles...).
		SaveX(ctx)

	expectedJobParams := `{"exampleParam":"Hello World"}`
	// Run Tests
	t.Run("CreateWithoutFiles", newCreateJobTest(
		gqlClient,
		[]int{testSessions[0].ID, testSessions[1].ID},
		ent.CreateJobInput{
			Name:       "TestJob",
			Parameters: &expectedJobParams,
			TomeID:     testTome.ID,
		},
		func(t *testing.T, id int, err error) {
			require.NoError(t, err)
			require.NotZero(t, id)

			// Ensure job was created with proper fields
			job := graph.Job.GetX(ctx, id)
			assert.Equal(t, "TestJob", job.Name)
			assert.Equal(t, `{"exampleParam":"Hello World"}`, job.Parameters)

			// Ensure tome edge was set
			tomeID := job.QueryTome().OnlyIDX(ctx)
			assert.Equal(t, testTome.ID, tomeID)

			// Ensure no bundle was created
			assert.Empty(t, job.QueryTome().QueryFiles().AllX(ctx))
			assert.False(t, job.QueryBundle().ExistX(ctx))

			// Ensure tasks were created properly
			tasks := job.QueryTasks().AllX(ctx)
			assert.Len(t, tasks, 2)
			for i, task := range tasks {
				assert.WithinRange(t, task.CreatedAt, time.Now().Add(-1*time.Second), time.Now().Add(1*time.Second))
				assert.WithinRange(t, task.LastModifiedAt, time.Now().Add(-1*time.Second), time.Now().Add(1*time.Second))
				assert.Zero(t, task.ClaimedAt)
				assert.Zero(t, task.ExecStartedAt)
				assert.Zero(t, task.ExecFinishedAt)
				assert.Empty(t, task.Output)
				assert.Empty(t, task.Error)
				assert.Equal(t, job.ID, task.QueryJob().OnlyIDX(ctx))
				assert.Equal(t, testSessions[i].ID, task.QuerySession().OnlyIDX(ctx))
			}
		},
	))
	t.Run("CreateWithFiles", newCreateJobTest(
		gqlClient,
		[]int{testSessions[0].ID, testSessions[1].ID},
		ent.CreateJobInput{
			Name:   "TestJobWithFiles",
			TomeID: testTomeWithFiles.ID,
		},
		func(t *testing.T, id int, err error) {
			require.NoError(t, err)
			require.NotZero(t, id)

			// Ensure job was created with proper fields
			job := graph.Job.GetX(ctx, id)
			assert.Equal(t, "TestJobWithFiles", job.Name)

			// Ensure bundle was created properly
			bundle := job.QueryBundle().OnlyX(ctx)
			gr, err := gzip.NewReader(bytes.NewReader(bundle.Content))
			require.NoError(t, err)
			tarReader := tar.NewReader(gr)

			// Read files from bundle and ensure they're correct
			for i, testFile := range testFiles {
				hdr, err := tarReader.Next()
				require.NoError(t, err, "failed to read file header %q (index=%d)", testFile.Name, i)
				assert.Equal(t, testFile.Name, hdr.Name)
				fileContent, err := io.ReadAll(tarReader)
				require.NoError(t, err, "failed to read file %q (index=%d)", testFile.Name, i)
				assert.Equal(t, string(testFile.Content), string(fileContent))
			}
			_, readerErr := tarReader.Next()
			assert.ErrorIs(t, readerErr, io.EOF) // Ensure these are the only files present

			// Ensure tome edge was set
			tomeID := job.QueryTome().OnlyIDX(ctx)
			assert.Equal(t, testTomeWithFiles.ID, tomeID)

			// Ensure tasks were created properly
			tasks := job.QueryTasks().AllX(ctx)
			assert.Len(t, tasks, 2)
			for i, task := range tasks {
				assert.WithinRange(t, task.CreatedAt, time.Now().Add(-1*time.Second), time.Now().Add(1*time.Second))
				assert.WithinRange(t, task.LastModifiedAt, time.Now().Add(-1*time.Second), time.Now().Add(1*time.Second))
				assert.Zero(t, task.ClaimedAt)
				assert.Zero(t, task.ExecStartedAt)
				assert.Zero(t, task.ExecFinishedAt)
				assert.Empty(t, task.Output)
				assert.Empty(t, task.Error)
				assert.Equal(t, job.ID, task.QueryJob().OnlyIDX(ctx))
				assert.Equal(t, testSessions[i].ID, task.QuerySession().OnlyIDX(ctx))
			}
		},
	))
}

func newCreateJobTest(gqlClient *client.Client, sessionIDs []int, input ent.CreateJobInput, checks ...func(t *testing.T, id int, err error)) func(t *testing.T) {
	return func(t *testing.T) {
		// Define the mutatation for testing, taking the input as a variable
		mut := `mutation newCreateJobTest($sessionIDs: [ID!]!, $input: CreateJobInput!) { createJob(sessionIDs:$sessionIDs, input:$input) {
			id
			tasks {
				id
			}
		} }`

		// Make our request to the GraphQL API
		var resp struct {
			CreateJob struct {
				ID    string
				Tasks []struct {
					ID string
				} `json:"tasks"`
			}
		}
		err := gqlClient.Post(mut, &resp,
			client.Var("sessionIDs", sessionIDs),
			client.Var("input", map[string]interface{}{
				"name":       input.Name,
				"parameters": input.Parameters,
				"tomeID":     input.TomeID,
			}),
		)

		// Run checks with error (if any) and resulting id
		for _, check := range checks {
			check(t, convertID(resp.CreateJob.ID), err)
		}
	}
}
