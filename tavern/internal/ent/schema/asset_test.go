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
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	var (
		expectedName    = "TestAsset"
		expectedContent = []byte("ABunchOfBytes")
		expectedSize    = 13
		expectedHash    = "adaf38cc9a3d8d810f051a0098cb8737001394ae9b85d9f6fa56dbc2bcc08db6"
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
}

func TestMultipleTomes(t *testing.T) {
	ctx := context.Background()
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
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
