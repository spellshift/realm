package main

import (
	"bytes"
	"context"
	"encoding/json"
	"image/png"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/ent/host"
	"realm.pub/tavern/internal/ent/tome"
)

// TestCreateTestData ensures createTestData runs without error and creates at least one beacon.
func TestCreateTestData(t *testing.T) {
	ctx := context.Background()
	graph := enttest.OpenTempDB(t)
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

	t.Run("ScreenshotsTest", func(t *testing.T) {
		firstHost := graph.Host.Query().Where(host.Name("Group 1 - Service-1")).OnlyX(ctx)
		screenshots := firstHost.QueryScreenshots().AllX(ctx)
		require.Len(t, screenshots, 55)
		contentSizes := make(map[uint64]struct{})
		for _, screenshot := range screenshots {
			_, err := png.DecodeConfig(bytes.NewReader(screenshot.Content))
			require.NoError(t, err)
			require.Equal(t, uint64(len(screenshot.Content)), screenshot.Size)
			contentSizes[screenshot.Size] = struct{}{}
		}
		require.Greater(t, len(contentSizes), 1, "test images should exercise sorting by size")

		secondHost := graph.Host.Query().Where(host.Name("Group 1 - Service-2")).OnlyX(ctx)
		require.Equal(t, 3, secondHost.QueryScreenshots().CountX(ctx))
	})
}
