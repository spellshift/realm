package tomes_test

import (
	"context"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/go-git/go-git/v5"
	"github.com/go-git/go-git/v5/plumbing/object"
	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/ent/tome"
	"realm.pub/tavern/tomes"
)

func TestImportFromRepo(t *testing.T) {
	// Create a temporary directory for the Git repository
	tmpDir, err := os.MkdirTemp("", "temprepo")
	require.NoError(t, err)
	defer os.RemoveAll(tmpDir)

	// Initialize a Git repository in the temporary directory
	_, err = git.PlainInit(tmpDir, false)
	require.NoError(t, err)

	// Create a dummy file and commit it
	err = os.MkdirAll(filepath.Join(tmpDir, "example"), 0755)
	require.NoError(t, err)
	err = os.WriteFile(filepath.Join(tmpDir, "example", "metadata.yml"), []byte("name: example\ndescription: An example tome!\nauthor: test\ntactic: DEFENSE_EVASION\nparamdefs:\n  - name: msg\n    label: Message\n    type: string\n    placeholder: Something to print\n"), 0644)
	require.NoError(t, err)
	err = os.WriteFile(filepath.Join(tmpDir, "example", "main.eldritch"), []byte("print(input_params['msg'])"), 0644)
	require.NoError(t, err)
	err = os.WriteFile(filepath.Join(tmpDir, "example", "file.txt"), []byte("This file exists"), 0644)
	require.NoError(t, err)

	repo, err := git.PlainOpen(tmpDir)
	require.NoError(t, err)

	w, err := repo.Worktree()
	require.NoError(t, err)

	_, err = w.Add(".")
	require.NoError(t, err)

	_, err = w.Commit("Initial commit", &git.CommitOptions{
		Author: &object.Signature{
			Name:  "Test",
			Email: "test@example.com",
			When:  time.Now(),
		},
	})
	require.NoError(t, err)

	ctx := context.Background()
	var (
		driverName     = "sqlite3"
		dataSourceName = "file:ent?mode=memory&cache=shared&_fk=1"
	)
	graph := enttest.Open(t, driverName, dataSourceName, enttest.WithOptions())
	defer graph.Close()

	// Initialize Git Importer
	gitImporter := tomes.NewGitImporter(graph)

	// Create repository
	repoEnt := graph.Repository.Create().SetURL(tmpDir).SaveX(ctx)
	repoEnt.URL = tmpDir // Override schema hook to test with local repo

	assert.NotEmpty(t, repoEnt.PrivateKey)

	filter := func(path string) bool {
		return strings.Contains(path, "example")
	}
	err = gitImporter.Import(ctx, repoEnt, filter)
	require.NoError(t, err)

	testTome := graph.Tome.Query().
		Where(tome.NameContains("example")).
		OnlyX(ctx)
	require.NotNil(t, testTome)
	assert.Equal(t, "print(input_params['msg'])", strings.TrimSpace(testTome.Eldritch))
	assert.Equal(t, `An example tome!`, testTome.Description)
	assert.Equal(t, `[{"name":"msg","label":"Message","type":"string","placeholder":"Something to print"}]`, testTome.ParamDefs)
	testTomeAssets, err := testTome.QueryAssets().All(ctx)
	assert.NoError(t, err)
	assert.Len(t, testTomeAssets, 1)
	assert.Equal(t, []byte("This file exists"), testTomeAssets[0].Content)
}
