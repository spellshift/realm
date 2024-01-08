package schema_test

import (
	"context"
	"testing"
	"time"

	"github.com/kcarretto/realm/tavern/internal/ent"
	"github.com/kcarretto/realm/tavern/internal/ent/enttest"
	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
)

func TestFileHooks(t *testing.T) {
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	var (
		expectedName    = "TestFile"
		expectedContent = []byte("ABunchOfBytes")
		expectedSize    = 13
		expectedHash    = "adaf38cc9a3d8d810f051a0098cb8737001394ae9b85d9f6fa56dbc2bcc08db6"
	)

	testFile := newFile(graph, expectedName, expectedContent)
	assert.NotNil(t, testFile)
	assert.NotZero(t, testFile.ID)
	assert.Equal(t, expectedName, testFile.Name)
	assert.Equal(t, string(expectedContent), string(testFile.Content))
	assert.Equal(t, expectedSize, testFile.Size)
	assert.Equal(t, expectedHash, testFile.Hash)
	assert.NotZero(t, testFile.CreatedAt)
	assert.WithinRange(t, testFile.CreatedAt, time.Now().Add(-1*time.Second), time.Now().Add(1*time.Second))
	assert.WithinRange(t, testFile.LastModifiedAt, time.Now().Add(-1*time.Second), time.Now().Add(1*time.Second))
	assert.NotZero(t, testFile.LastModifiedAt)
}

// newFile is a helper to create files directly via ent
func newFile(graph *ent.Client, name string, content []byte) *ent.File {
	return graph.File.Create().
		SetName(name).
		SetContent(content).
		SaveX(context.Background())
}
