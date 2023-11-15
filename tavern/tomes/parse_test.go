package tomes_test

import (
	"context"
	"testing"

	"github.com/kcarretto/realm/tavern/internal/ent/enttest"
	"github.com/kcarretto/realm/tavern/internal/ent/tome"
	"github.com/kcarretto/realm/tavern/tomes"
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
	assert.Equal(t, `print(input_params['msg'])`, testTome.Eldritch)
	assert.Equal(t, `An example tome!`, testTome.Description)
	assert.Equal(t, `[{"name":"msg","label":"Message","type":"string","placeholder":"Something to print"}]`, testTome.ParamDefs)
	testTomeFiles, err := testTome.Files(ctx)
	assert.NoError(t, err)
	assert.Len(t, testTomeFiles, 1)
	assert.Equal(t, "example/linux/test-file", testTomeFiles[0].Name)
	assert.Equal(t, []byte("This file exists"), testTomeFiles[0].Content)
}
