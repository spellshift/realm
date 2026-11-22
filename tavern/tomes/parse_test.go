package tomes_test

import (
	"context"
	"encoding/json"
	"strings"
	"testing"
	"testing/fstest"

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
	graph := enttest.OpenTempDB(t)
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
	testTomeAssets, err := testTome.QueryAssets().All(ctx)
	assert.NoError(t, err)
	assert.Len(t, testTomeAssets, 1)
	assert.Equal(t, "example/linux/test-file", testTomeAssets[0].Name)
	assert.Equal(t, []byte("This file exists"), testTomeAssets[0].Content)

	// Assert our Shell Aliases RC tome is uploaded with the SHELL_RC tactic
	rcTome := graph.Tome.Query().
		Where(tome.Name("Shell Aliases")).
		OnlyX(ctx)
	require.NotNil(t, rcTome)
	assert.Equal(t, tome.TacticSHELL_RC, rcTome.Tactic)
	assert.Contains(t, rcTome.Eldritch, "def ls(")

	installTome := graph.Tome.Query().
		Where(tome.Name("Install Service")).
		OnlyX(ctx)
	assert.Equal(t, tome.TacticPERSISTENCE, installTome.Tactic)
	assert.Contains(t, installTome.Eldritch, "parse_and_persist([config])")
	var installParams []tomes.ParamDefinition
	require.NoError(t, json.Unmarshal([]byte(installTome.ParamDefs), &installParams))
	require.Len(t, installParams, 5)
	assert.Equal(t, []string{"name", "description", "executable_name", "executable_args", "windows_display_name"}, []string{
		installParams[0].Name, installParams[1].Name, installParams[2].Name, installParams[3].Name, installParams[4].Name,
	})
}

func TestUploadTomes_InvalidTomeDoesNotSkipOthers(t *testing.T) {
	ctx := context.Background()
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	// "a_bad" sorts before "b_good"; its invalid support_model makes creation fail.
	fsys := fstest.MapFS{
		"a_bad/metadata.yml":   &fstest.MapFile{Data: []byte("name: Bad Tome\nauthor: test\ndescription: bad\ntactic: UNSPECIFIED\nsupport_model: BOGUS\n")},
		"a_bad/main.eldritch":  &fstest.MapFile{Data: []byte("print('bad')")},
		"b_good/metadata.yml":  &fstest.MapFile{Data: []byte("name: Good Tome\nauthor: test\ndescription: good\ntactic: UNSPECIFIED\nsupport_model: FIRST_PARTY\n")},
		"b_good/main.eldritch": &fstest.MapFile{Data: []byte("print('good')")},
	}

	// The invalid tome should be reported as an error.
	assert.Error(t, tomes.UploadTomes(ctx, graph, fsys))

	// The valid tome must still be loaded despite the earlier invalid one.
	loaded, err := graph.Tome.Query().Where(tome.Name("Good Tome")).Exist(ctx)
	require.NoError(t, err)
	assert.True(t, loaded, "good tome was not loaded after a bad tome")
}
