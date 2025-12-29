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
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/graphql"
	tavernhttp "realm.pub/tavern/internal/http"
	"realm.pub/tavern/tomes"
)

// TestCreateQuest ensures the createQuest mutation functions as expected
func TestCreateQuest(t *testing.T) {
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
	testHost := graph.Host.Create().
		SetIdentifier("ABCDEFG").
		SetPlatform(c2pb.Host_PLATFORM_UNSPECIFIED).
		SaveX(ctx)
	testBeacons := []*ent.Beacon{
		graph.Beacon.Create().
			SetHost(testHost).
			SetTransport(c2pb.ActiveTransport_TRANSPORT_UNSPECIFIED).
			SaveX(ctx),
		graph.Beacon.Create().
			SetHost(testHost).
			SetTransport(c2pb.ActiveTransport_TRANSPORT_UNSPECIFIED).
			SaveX(ctx),
	}
	testTome := graph.Tome.Create().
		SetName("Test Tome").
		SetDescription("Ensures the world feels greeted").
		SetAuthor("kcarretto").
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
		SetAuthor("kcarretto").
		SetEldritch(`print("Hello World!")`).
		AddFiles(testFiles...).
		SaveX(ctx)

	expectedQuestParams := `{"exampleParam":"Hello World"}`
	// Run Tests
	t.Run("CreateWithoutFiles", newCreateQuestTest(
		gqlClient,
		[]int{testBeacons[0].ID, testBeacons[1].ID},
		ent.CreateQuestInput{
			Name:       "TestQuest",
			Parameters: &expectedQuestParams,
			TomeID:     testTome.ID,
		},
		func(t *testing.T, id int, err error) {
			require.NoError(t, err)
			require.NotZero(t, id)

			// Ensure quest was created with proper fields
			quest := graph.Quest.GetX(ctx, id)
			assert.Equal(t, "TestQuest", quest.Name)
			assert.Equal(t, `{"exampleParam":"Hello World"}`, quest.Parameters)

			// Ensure tome edge was set
			tomeID := quest.QueryTome().OnlyIDX(ctx)
			assert.Equal(t, testTome.ID, tomeID)

			// Ensure no bundle was created
			assert.Empty(t, quest.QueryTome().QueryFiles().AllX(ctx))
			assert.False(t, quest.QueryBundle().ExistX(ctx))

			// Ensure tasks were created properly
			tasks := quest.QueryTasks().AllX(ctx)
			assert.Len(t, tasks, 2)
			for i, task := range tasks {
				assert.WithinRange(t, task.CreatedAt, time.Now().Add(-1*time.Second), time.Now().Add(1*time.Second))
				assert.WithinRange(t, task.LastModifiedAt, time.Now().Add(-1*time.Second), time.Now().Add(1*time.Second))
				assert.Zero(t, task.ClaimedAt)
				assert.Zero(t, task.ExecStartedAt)
				assert.Zero(t, task.ExecFinishedAt)
				assert.Empty(t, task.Output)
				assert.Empty(t, task.Error)
				assert.Equal(t, quest.ID, task.QueryQuest().OnlyIDX(ctx))
				assert.Equal(t, testBeacons[i].ID, task.QueryBeacon().OnlyIDX(ctx))
			}
		},
	))
	t.Run("CreateWithFiles", newCreateQuestTest(
		gqlClient,
		[]int{testBeacons[0].ID, testBeacons[1].ID},
		ent.CreateQuestInput{
			Name:   "TestQuestWithFiles",
			TomeID: testTomeWithFiles.ID,
		},
		func(t *testing.T, id int, err error) {
			require.NoError(t, err)
			require.NotZero(t, id)

			// Ensure quest was created with proper fields
			quest := graph.Quest.GetX(ctx, id)
			assert.Equal(t, "TestQuestWithFiles", quest.Name)

			// Ensure bundle was created properly
			bundle := quest.QueryBundle().OnlyX(ctx)
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
			tomeID := quest.QueryTome().OnlyIDX(ctx)
			assert.Equal(t, testTomeWithFiles.ID, tomeID)

			// Ensure tasks were created properly
			tasks := quest.QueryTasks().AllX(ctx)
			assert.Len(t, tasks, 2)
			for i, task := range tasks {
				assert.WithinRange(t, task.CreatedAt, time.Now().Add(-1*time.Second), time.Now().Add(1*time.Second))
				assert.WithinRange(t, task.LastModifiedAt, time.Now().Add(-1*time.Second), time.Now().Add(1*time.Second))
				assert.Zero(t, task.ClaimedAt)
				assert.Zero(t, task.ExecStartedAt)
				assert.Zero(t, task.ExecFinishedAt)
				assert.Empty(t, task.Output)
				assert.Empty(t, task.Error)
				assert.Equal(t, quest.ID, task.QueryQuest().OnlyIDX(ctx))
				assert.Equal(t, testBeacons[i].ID, task.QueryBeacon().OnlyIDX(ctx))
			}
		},
	))
}

func newCreateQuestTest(gqlClient *client.Client, beaconIDs []int, input ent.CreateQuestInput, checks ...func(t *testing.T, id int, err error)) func(t *testing.T) {
	return func(t *testing.T) {
		// Define the mutatation for testing, taking the input as a variable
		mut := `mutation newCreateQuestTest($beaconIDs: [ID!]!, $input: CreateQuestInput!) { createQuest(beaconIDs:$beaconIDs, input:$input) {
			id
			tasks {
				edges {
					node {
						id
					}
				}
			}
		} }`

		// Make our request to the GraphQL API
		var resp struct {
			CreateQuest struct {
				ID    string
				Tasks struct {
					Edges []struct {
						Node struct {
							ID string
						} `json:"node"`
					} `json:"edges"`
				} `json:"tasks"`
			}
		}
		err := gqlClient.Post(mut, &resp,
			client.Var("beaconIDs", beaconIDs),
			client.Var("input", map[string]interface{}{
				"name":       input.Name,
				"parameters": input.Parameters,
				"tomeID":     input.TomeID,
			}),
		)

		// Run checks with error (if any) and resulting id
		for _, check := range checks {
			check(t, convertID(resp.CreateQuest.ID), err)
		}
	}
}
