package schema_test

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"

	_ "github.com/mattn/go-sqlite3"
)

// TestCreateLinkWithExplicitExpiresAt verifies that explicitly setting expiresAt works correctly.
func TestCreateLinkWithExplicitExpiresAt(t *testing.T) {
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	ctx := context.Background()

	// Create an asset first
	asset := graph.Asset.Create().
		SetName("TestAsset2").
		SetContent([]byte("test content")).
		SaveX(ctx)

	// Create a link with explicit expiresAt
	futureTime := time.Now().Add(24 * time.Hour)
	downloadsRemaining := 5
	input := ent.CreateLinkInput{
		AssetID:            asset.ID,
		DownloadsRemaining: &downloadsRemaining,
		ExpiresAt:          &futureTime,
	}

	link, err := graph.Link.Create().SetInput(input).Save(ctx)
	require.NoError(t, err)

	// Verify the explicit expiresAt was used
	assert.WithinDuration(t, futureTime, link.ExpiresAt, time.Second)
	assert.Equal(t, 5, link.DownloadsRemaining)
}
