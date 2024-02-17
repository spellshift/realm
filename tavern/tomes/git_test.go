package tomes_test

import (
	"context"
	"strings"
	"testing"

	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/ent/tome"
	"realm.pub/tavern/tomes"
)

const RealmGIT = "../../"

func TestImportFromRepo(t *testing.T) {
	ctx := context.Background()
	var (
		driverName     = "sqlite3"
		dataSourceName = "file:ent?mode=memory&cache=shared&_fk=1"
	)
	graph := enttest.Open(t, driverName, dataSourceName, enttest.WithOptions())
	defer graph.Close()

	filter := func(path string) bool {
		return strings.Contains(path, "tavern/tomes")
	}
	_, err := tomes.ImportFromRepo(ctx, graph, RealmGIT, filter)
	require.NoError(t, err)

	testTome := graph.Tome.Query().
		Where(tome.NameContains("Example")).
		OnlyX(ctx)
	require.NotNil(t, testTome)
	assert.Equal(t, "print(input_params['msg'])", strings.TrimSpace(testTome.Eldritch))
	assert.Equal(t, `An example tome!`, testTome.Description)
	assert.Equal(t, `[{"name":"msg","label":"Message","type":"string","placeholder":"Something to print"}]`, testTome.ParamDefs)
	testTomeFiles, err := testTome.Files(ctx)
	assert.NoError(t, err)
	assert.Len(t, testTomeFiles, 1)
	assert.Equal(t, []byte("This file exists"), testTomeFiles[0].Content)
}
