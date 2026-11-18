package managed

import (
	"context"
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestFileStoreDoesNotOverwriteIdentity(t *testing.T) {
	ctx := context.Background()
	store := fileStore{path: filepath.Join(t.TempDir(), "identity.json")}
	identity := &Identity{ID: "first"}
	require.NoError(t, store.Save(ctx, identity))
	require.ErrorIs(t, store.Save(ctx, &Identity{ID: "second"}), os.ErrExist)
	loaded, err := store.Load(ctx)
	require.NoError(t, err)
	require.Equal(t, "first", loaded.ID)
	require.NoError(t, os.Chmod(store.path, 0644))
	_, err = store.Load(ctx)
	require.ErrorContains(t, err, "permissions are too broad")
}
