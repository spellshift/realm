package tomes_test

import (
	"context"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/ent/tome"
	"realm.pub/tavern/tomes"

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
		Where(tome.NameContains("Example")).
		OnlyX(ctx)
	require.NotNil(t, testTome)
	assert.Equal(t, "print(input_params['msg'])", strings.TrimSpace(testTome.Eldritch))
	assert.Equal(t, `An example tome!`, testTome.Description)
	assert.Equal(t, `[{"name":"msg","label":"Message","type":"string","placeholder":"Something to print"}]`, testTome.ParamDefs)
	testTomeFiles, err := testTome.QueryFiles().All(ctx)
	assert.NoError(t, err)
	assert.Len(t, testTomeFiles, 1)
	assert.Equal(t, "example/linux/test-file", testTomeFiles[0].Name)
	assert.Equal(t, []byte("This file exists"), testTomeFiles[0].Content)
}
