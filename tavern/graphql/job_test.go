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
	"github.com/kcarretto/realm/tavern/ent"
	"github.com/kcarretto/realm/tavern/ent/enttest"
	"github.com/kcarretto/realm/tavern/graphql"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestCreateJob ensures the createJob mutation functions as expected
func TestCreateJob(t *testing.T) {
	// Initialize Test Context
	ctx := context.Background()

	// Initialize DB Backend
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	// Initialize sample data
	testTargets := []*ent.Target{
		graph.Target.Create().
			SetName("target1").
			SaveX(ctx),
		graph.Target.Create().
			SetName("target2").
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

	// Create a new GraphQL server (needed for auth middleware)
	srv := handler.NewDefaultServer(graphql.NewSchema(graph))

	// Create a new GraphQL client (connected to our http server)
	gqlClient := client.New(srv)

	// Run Tests
	t.Run("CreateWithoutFiles", newCreateJobTest(
		gqlClient,
		[]int{testTargets[0].ID, testTargets[1].ID},
		ent.CreateJobInput{
			Name:   "TestJob",
			TomeID: testTome.ID,
		},
		func(t *testing.T, id int, err error) {
			require.NoError(t, err)
			require.NotZero(t, id)

			// Ensure job was created with proper fields
			job := graph.Job.GetX(ctx, id)
			assert.Equal(t, "TestJob", job.Name)

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
				assert.Equal(t, testTargets[i].ID, task.QueryTarget().OnlyIDX(ctx))
			}
		},
	))
	t.Run("CreateWithFiles", newCreateJobTest(
		gqlClient,
		[]int{testTargets[0].ID, testTargets[1].ID},
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
				assert.Equal(t, testTargets[i].ID, task.QueryTarget().OnlyIDX(ctx))
			}
		},
	))
}

func newCreateJobTest(gqlClient *client.Client, targetIDs []int, input ent.CreateJobInput, checks ...func(t *testing.T, id int, err error)) func(t *testing.T) {
	return func(t *testing.T) {
		// Define the mutatation for testing, taking the input as a variable
		mut := `mutation newCreateJobTest($targetIDs: [ID!]!, $input: CreateJobInput!) { createJob(targetIDs:$targetIDs, input:$input) { id } }`

		// Make our request to the GraphQL API
		var resp struct {
			CreateJob struct{ ID string }
		}
		err := gqlClient.Post(mut, &resp,
			client.Var("targetIDs", targetIDs),
			client.Var("input", map[string]interface{}{
				"name":   input.Name,
				"tomeID": input.TomeID,
			}),
		)

		// Run checks with error (if any) and resulting id
		for _, check := range checks {
			check(t, convertID(resp.CreateJob.ID), err)
		}
	}
}
