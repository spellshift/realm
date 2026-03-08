package main

import (
	"context"
	"encoding/json"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/ent/tome"
)

// TestCreateTestData ensures createTestData runs without error and creates at least one beacon.
func TestCreateTestData(t *testing.T) {
	var (
		ctx            = context.Background()
		driverName     = "sqlite3"
		dataSourceName = "file:test-create-test-data?mode=memory&cache=shared&_fk=1"
	)
	graph := enttest.Open(t, driverName, dataSourceName, enttest.WithOptions())
	defer graph.Close()

	createTestData(ctx, graph)
	assert.True(t, graph.Beacon.Query().ExistX(ctx))

	// Ensure ParamDefs hasn't changed
	t.Run("ParamDefsTest", func(t *testing.T) {
		exampleTome := graph.Tome.Query().
			Where(tome.ParamDefsContains("[")).
			FirstX(ctx)
		require.NotNil(t, exampleTome)

		var paramDefs = []struct {
			Name        string `json:"name"`
			Label       string `json:"label"`
			Type        string `json:"type"`
			Placeholder string `json:"placeholder"`
		}{}
		require.NoError(t, json.Unmarshal([]byte(exampleTome.ParamDefs), &paramDefs))
		require.Greater(t, len(paramDefs), 0)
		assert.NotEmpty(t, paramDefs[0].Name)

	})

	t.Run("ProcessListTest", func(t *testing.T) {
		count := graph.HostProcess.Query().CountX(ctx)
		// We expect 12 hosts (3 groups * 4 platforms) to have processes.
		// Each platform has a specific number of processes.
		// Windows: 11
		// Linux: 10
		// MacOS: 8
		// BSD: 7
		// Total per group: 11 + 10 + 8 + 7 = 36
		// Total for 3 groups: 36 * 3 = 108
		assert.Equal(t, 108, count)
	})
}
