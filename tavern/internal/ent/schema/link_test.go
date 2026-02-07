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

// TestCreateLinkWithDownloadsRemainingDefaultExpiresAt tests that creating a link
// with only downloadsRemaining set (and no expiresAt) results in a MySQL-compatible
// default expires_at value.
//
// This test reproduces GitHub issue #1584:
// When calling createLink mutation with downloadsRemaining set, the default
// expires_at value of 1970-01-01 (Unix epoch 0) causes MySQL to fail with:
// "incorrect date time value '1970-01-01' for column 'expires_at' at row 1"
//
// MySQL's minimum DATETIME value is '1000-01-01 00:00:00', but values near the
// Unix epoch (1970-01-01) can cause issues with timezone conversions that result
// in negative timestamps, which MySQL rejects.
func TestCreateLinkWithDownloadsRemainingDefaultExpiresAt(t *testing.T) {
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	ctx := context.Background()

	// Create an asset first (required for link creation)
	asset := graph.Asset.Create().
		SetName("TestAsset").
		SetContent([]byte("test content")).
		SaveX(ctx)

	// Create a link with only downloadsRemaining set (simulating the GraphQL mutation)
	// This is the problematic case from issue #1584
	downloadsRemaining := 1
	input := ent.CreateLinkInput{
		AssetID:            asset.ID,
		DownloadsRemaining: &downloadsRemaining,
		// ExpiresAt is intentionally NOT set to trigger the default value
	}

	link, err := graph.Link.Create().SetInput(input).Save(ctx)
	require.NoError(t, err, "Link creation should succeed")

	// The bug: the default expires_at is time.Unix(0, 0) which equals 1970-01-01 00:00:00 UTC
	// This value is rejected by MySQL with the error:
	// "incorrect date time value '1970-01-01' for column 'expires_at'"
	//
	// MySQL minimum valid DATETIME is '1000-01-01 00:00:00'
	// However, timestamps near epoch 0 (1970-01-01) can cause issues with timezones
	// that result in negative Unix timestamps, which MySQL rejects.
	//
	// The default should NOT be Unix epoch 0 (1970-01-01).
	epochTime := time.Unix(0, 0)
	mysqlMinTime := time.Date(1000, 1, 1, 0, 0, 0, 0, time.UTC)

	// This test will FAIL if the bug is present (default is epoch 0)
	// and PASS once the fix is applied
	if link.ExpiresAt.Equal(epochTime) {
		t.Errorf("BUG DETECTED (GitHub #1584): Default expires_at is Unix epoch 0 (1970-01-01 00:00:00 UTC), "+
			"which MySQL rejects with error: 'incorrect date time value'. "+
			"Current value: %v. The default should be a MySQL-compatible value >= %v",
			link.ExpiresAt.UTC(), mysqlMinTime)
	}

	// Verify the default expires_at is MySQL-compatible (>= 1000-01-01)
	assert.True(t, link.ExpiresAt.After(mysqlMinTime) || link.ExpiresAt.Equal(mysqlMinTime),
		"Default expires_at (%v) must be >= MySQL minimum datetime (%v)",
		link.ExpiresAt, mysqlMinTime)

	// Verify the link was created with the correct downloadsRemaining
	assert.Equal(t, 1, link.DownloadsRemaining)
}

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
