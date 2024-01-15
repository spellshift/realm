package c2_test

import (
	"context"
	"database/sql"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"realm.pub/tavern/internal/c2"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent/enttest"

	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"gopkg.in/yaml.v3"
)

type testRequest struct {
	Method        string         `yaml:"method"`
	Request       map[string]any `yaml:"request"`
	Expected      map[string]any `yaml:"expected"`
	ExpectedError string         `yaml:"expected_error"`
}

type testCase struct {
	State    string        `yaml:"state"`
	Requests []testRequest `yaml:"requests"`
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
	srv := c2.New(graph)

	// Iterate through requests
	for i, tr := range tc.Requests {

		t.Run(fmt.Sprintf("%d-%s", i, tr.Method), func(t *testing.T) {
			// Marshal expected result to JSON
			expectedJSON, err := json.Marshal(tr.Expected)
			require.NoError(t, err)

			// Marshal request to JSON
			requestJSON, err := json.Marshal(tr.Request)
			require.NoError(t, err)

			// For each method:
			// 	- Unmarshal request JSON into input type
			//	- Call method
			//	- Check for expected error
			// 	- Marshal result to JSON and compare with expected (if there is output)
			switch tr.Method {
			case "ClaimTasks":
				req := c2pb.ClaimTasksRequest{}
				require.NoError(t, json.Unmarshal(requestJSON, &req))
				result, resultErr := srv.ClaimTasks(context.Background(), &req)
				if tr.ExpectedError != "" {
					assert.ErrorContains(t, resultErr, tr.ExpectedError)
					return
				}
				assert.NoError(t, resultErr)
				resultJSON, err := json.Marshal(result)
				require.NoError(t, err, "failed to marshal result to json")
				assert.Equal(t, string(expectedJSON), string(resultJSON), "response does not match expected result")
			case "ReportTaskOutput":
				req := c2pb.ReportTaskOutputRequest{}
				require.NoError(t, json.Unmarshal(requestJSON, &req))
				_, resultErr := srv.ReportTaskOutput(context.Background(), &req)
				if tr.ExpectedError != "" {
					assert.ErrorContains(t, resultErr, tr.ExpectedError)
					return
				}
				assert.NoError(t, resultErr)
			default:
				t.Fatalf("invalid method name: %q", tr.Method)
			}
		})
	}
}

// TestAPI finds and runs all test cases defined in the testdata directory.
func TestAPI(t *testing.T) {
	runTestsInDir(t, "testdata")
}

func runTestsInDir(t *testing.T, root string) {
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
