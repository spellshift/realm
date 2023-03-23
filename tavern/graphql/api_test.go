package graphql_test

import (
	"database/sql"
	"encoding/json"
	"io/fs"
	"net/http"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/kcarretto/realm/tavern/auth"
	"github.com/kcarretto/realm/tavern/ent/enttest"
	"github.com/kcarretto/realm/tavern/graphql"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/99designs/gqlgen/client"
	"github.com/99designs/gqlgen/graphql/handler"
	_ "github.com/mattn/go-sqlite3"
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

func runTestCase(t *testing.T, path string) {
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
	srv := auth.Middleware(handler.NewDefaultServer(graphql.NewSchema(graph)), graph)
	gqlClient := client.New(srv)

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
	require.Nil(t, queryErr, "query failed with error")

	// Marshal response to JSON
	respJSON, err := json.Marshal(resp)
	require.NoError(t, err, "failed to marshal response to JSON")

	// Assert the result is as expected
	assert.Equal(t, string(expectedJSON), string(respJSON), "response does not match expected result")
}

// TestAPI finds and runs all test cases defined in the testdata directory.
func TestAPI(t *testing.T) {
	filepath.Walk("testdata", func(path string, info fs.FileInfo, err error) error {
		if info.IsDir() || err != nil || filepath.Ext(path) != ".yml" {
			return err
		}

		testName := strings.TrimSuffix(filepath.Base(path), ".yml")

		t.Run(testName, func(t *testing.T) {
			runTestCase(t, path)
		})
		return nil
	})
}
