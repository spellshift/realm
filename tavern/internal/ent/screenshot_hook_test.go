package ent_test

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	_ "github.com/mattn/go-sqlite3"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent/enttest"
)

func TestHookDeriveScreenshotInfo(t *testing.T) {
	client := enttest.OpenTempDB(t)
	defer client.Close()

	ctx := context.Background()

	h, err := client.Host.Create().
		SetIdentifier("screenshot-hook-host").
		SetPlatform(c2pb.Host_PLATFORM_MACOS).
		Save(ctx)
	require.NoError(t, err)

	content := []byte("initial-screenshot-payload")
	sc, err := client.Screenshot.Create().
		SetName("initial.png").
		SetContent(content).
		SetHost(h).
		Save(ctx)
	require.NoError(t, err)
	require.Equal(t, uint64(len(content)), sc.Size)
	require.NotEmpty(t, sc.Hash)

	origSize := sc.Size
	origHash := sc.Hash

	// Updating fields other than content must preserve size and hash.
	updated, err := client.Screenshot.UpdateOneID(sc.ID).
		SetName("renamed.png").
		Save(ctx)
	require.NoError(t, err)
	assert.Equal(t, "renamed.png", updated.Name)
	assert.Equal(t, origSize, updated.Size)
	assert.Equal(t, origHash, updated.Hash)

	// Updating content must recompute size and hash.
	newContent := []byte("updated-screenshot-payload-longer")
	updatedContent, err := client.Screenshot.UpdateOneID(sc.ID).
		SetContent(newContent).
		Save(ctx)
	require.NoError(t, err)
	assert.Equal(t, uint64(len(newContent)), updatedContent.Size)
	assert.NotEqual(t, origHash, updatedContent.Hash)
}
