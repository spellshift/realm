package tomes_test

import (
	"context"
	"testing"

	"github.com/kcarretto/realm/contrib/tomes"
	"github.com/kcarretto/realm/tavern/ent/enttest"
	"github.com/kcarretto/realm/tavern/ent/tome"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	_ "github.com/mattn/go-sqlite3"
)

func TestUploadTomes(t *testing.T) {
	// Setup
	ctx := context.Background()
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	// Assert our example tome is there
	require.NoError(t, tomes.UploadTomes(ctx, graph, tomes.FileSystem))
	testTome := graph.Tome.Query().
		Where(tome.Name("example")).
		OnlyX(ctx)
	require.NotNil(t, testTome)
	assert.Equal(t, `print("Hello World")`, testTome.Eldritch)
	assert.Equal(t, `An example tome!`, testTome.Description)
	testTomeFiles, err := testTome.Files(ctx)
	assert.NoError(t, err)
	assert.Len(t, testTomeFiles, 1)
	assert.Equal(t, "example/linux/test-implant", testTomeFiles[0].Name)
	assert.Equal(t, []byte("meowware"), testTomeFiles[0].Content)
}
