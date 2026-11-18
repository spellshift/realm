package schema_test

import (
	"context"
	"testing"
	"time"

	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
)

func TestAssetHooks(t *testing.T) {
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	var (
		expectedName    = "TestAsset"
		expectedContent = []byte("ABunchOfBytes")
		expectedSize    = 13
		expectedHash    = "49cf3b483e2bac1caa333bbd2ee76c889b998241"
	)

	testAsset := newAsset(graph, expectedName, expectedContent)
	assert.NotNil(t, testAsset)
	assert.NotZero(t, testAsset.ID)
	assert.Equal(t, expectedName, testAsset.Name)
	assert.Equal(t, string(expectedContent), string(testAsset.Content))
	assert.Equal(t, expectedSize, testAsset.Size)
	assert.Equal(t, expectedHash, testAsset.Hash)
	assert.NotZero(t, testAsset.CreatedAt)
	assert.WithinRange(t, testAsset.CreatedAt, time.Now().Add(-1*time.Second), time.Now().Add(1*time.Second))
	assert.WithinRange(t, testAsset.LastModifiedAt, time.Now().Add(-1*time.Second), time.Now().Add(1*time.Second))
	assert.NotZero(t, testAsset.LastModifiedAt)
	assert.Equal(t, "FINISHED", string(testAsset.Status))

	updated := graph.Asset.UpdateOne(testAsset).SetStatus("ERROR").SaveX(context.Background())
	assert.Equal(t, expectedSize, updated.Size)
	assert.Equal(t, expectedHash, updated.Hash)
	assert.Equal(t, expectedContent, updated.Content)
}

func TestMultipleTomes(t *testing.T) {
	ctx := context.Background()
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	assets := []*ent.Asset{
		newAsset(graph, "TestAsset001", []byte("Test Content")),
	}
	graph.Tome.Create().
		SetName("TestTome001").
		SetEldritch(`print("hello world")`).
		SetDescription("Hello World").
		SetAuthor("kcarretto").
		AddAssets(assets...).
		SaveX(ctx)
	graph.Tome.Create().
		SetName("TestTome002").
		SetEldritch(`print("hello world")`).
		SetDescription("Hello World").
		SetAuthor("kcarretto").
		AddAssets(assets...).
		SaveX(ctx)
}

// newAsset is a helper to create assets directly via ent
func newAsset(graph *ent.Client, name string, content []byte) *ent.Asset {
	return graph.Asset.Create().
		SetName(name).
		SetContent(content).
		SaveX(context.Background())
}
