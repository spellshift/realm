package tomes_test

import (
	"context"
	"path/filepath"
	"strings"
	"testing"

	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/ent/tome"
	"realm.pub/tavern/tomes"
)

func TestImportFromRepo(t *testing.T) {
	localGit := filepath.Join("..", "..", "..", "realm")
	ctx := context.Background()
	var (
		driverName     = "sqlite3"
		dataSourceName = "file:ent?mode=memory&cache=shared&_fk=1"
	)
	graph := enttest.Open(t, driverName, dataSourceName, enttest.WithOptions())
	defer graph.Close()

	// Initialize Git Importer
	git := tomes.NewGitImporter(graph)

	// Create repository
	repo := graph.Repository.Create().SetURL(localGit).SaveX(ctx)
	repo.URL = localGit // Override schema hook to test with local repo

	assert.NotEmpty(t, repo.PrivateKey)

	filter := func(path string) bool {
		return strings.Contains(path, "example")
	}
	err := git.Import(ctx, repo, filter)
	require.NoError(t, err)

	testTome := graph.Tome.Query().
		Where(tome.NameContains("example")).
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
