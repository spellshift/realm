package graphql_test

import (
	"context"
	"database/sql"
	"encoding/json"
	"net/http"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/ent/tome"
	"realm.pub/tavern/internal/graphql"
	tavernhttp "realm.pub/tavern/internal/http"

	"github.com/99designs/gqlgen/client"
	"github.com/99designs/gqlgen/graphql/handler"
	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"gopkg.in/yaml.v3"
)

type testCase struct {
	State     string `yaml:"state"`
	Requestor struct {
		SessionToken string `yaml:"session_token"`
	} `yaml:"requestor"`
	Query         string         `yaml:"query"`
	Variables     map[string]any `yaml:"variables"`
	Expected      map[string]any `yaml:"expected"`
	ExpectedError string         `yaml:"expected_error"`
}

type importerFake struct {
	graph *ent.Client
}

// Import fake imports a simple static tome for testing.
func (fake importerFake) Import(ctx context.Context, repo *ent.Repository, filters ...func(path string) bool) error {
	_, err := fake.graph.Tome.Create().
		SetName("expected_tome").
		SetDescription("expected_description").
		SetAuthor("expected_author").
		SetParamDefs("").
		SetSupportModel(tome.SupportModelCOMMUNITY).
		SetTactic(tome.TacticIMPACT).
		SetEldritch(`print("expected")`).
		SetRepository(repo).
		OnConflict().
		UpdateNewValues().
		ID(ctx)
	return err
}

func runTestCase(t *testing.T, path string) {
	t.Helper()

	// TestDB Config
	var (
		driverName     = "sqlite3"
		dataSourceName = "file:ent?mode=memory&cache=shared&_fk=1"
	)

	// Read Test Case
	tcBytes, err := os.ReadFile(path)
	require.NoError(t, err, "failed to read test case %q", path)

	// Parse Test Case
	var tc testCase
	yamlErr := yaml.Unmarshal(tcBytes, &tc)
	require.NoError(t, yamlErr, "failed to parse test case %q", path)

	// Marshal expected result to JSON
	expectedJSON, err := json.Marshal(tc.Expected)
	require.NoError(t, err)

	// Ent Client
	graph := enttest.Open(t, driverName, dataSourceName, enttest.WithOptions())
	defer graph.Close()

	// Initial DB State
	db, err := sql.Open(driverName, dataSourceName)
	require.NoError(t, err, "failed to open test db")
	defer db.Close()
	_, dbErr := db.Exec(tc.State)
	require.NoError(t, dbErr, "failed to setup test db state")

	// Server
	srv := tavernhttp.NewServer(
		tavernhttp.RouteMap{
			"/graphql": handler.NewDefaultServer(graphql.NewSchema(graph, importerFake{graph}, nil, nil)),
		},
		tavernhttp.WithAuthentication(graph),
	)
	gqlClient := client.New(srv, client.Path("/graphql"))

	var opts []client.Option

	// Variables
	for key, val := range tc.Variables {
		opts = append(opts, client.Var(key, val))
	}

	// Requestor
	if tc.Requestor.SessionToken != "" {
		opts = append(opts, client.AddCookie(&http.Cookie{
			Name:    auth.SessionCookieName,
			Value:   tc.Requestor.SessionToken,
			Expires: time.Now().Add(24 * time.Hour),
		}))
	}

	// Make Request
	resp := new(map[string]any)
	queryErr := gqlClient.Post(tc.Query, resp, opts...)

	// Handle Expected Errors
	if tc.ExpectedError != "" {
		assert.ErrorContains(t, queryErr, tc.ExpectedError)
		return
	}
	require.NoError(t, queryErr, "query failed with error")

	// Marshal response to JSON
	respJSON, err := json.Marshal(resp)
	require.NoError(t, err, "failed to marshal response to JSON")

	// Assert the result is as expected
	assert.Equal(t, string(expectedJSON), string(respJSON), "response does not match expected result")
}

// TestAPI finds and runs all test cases defined in the testdata directory.
func TestAPI(t *testing.T) {
	runTestsInDir(t, "testdata")
}

func runTestsInDir(t *testing.T, root string) {
	t.Helper()

	files, err := os.ReadDir(root)
	require.NoError(t, err)

	for _, f := range files {
		// Derive relative path
		path := filepath.Join(root, f.Name())

		// Recurse in subdirectories by grouping them into sub-tests
		if f.IsDir() {
			t.Run(filepath.Base(f.Name()), func(t *testing.T) {
				runTestsInDir(t, path)
			})
			continue
		}

		// Skip files that are not test case files
		if filepath.Ext(path) != ".yml" {
			continue
		}

		// Run test case
		testName := filepath.Base(strings.TrimSuffix(path, ".yml"))
		t.Run(testName, func(t *testing.T) {
			runTestCase(t, path)
		})
	}
}
