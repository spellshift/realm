package tomebundle

import (
	"context"
	"testing"

	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
)

func TestBundleIdentityIncludesNamesAndContents(t *testing.T) {
	ctx := context.Background()
	client := enttest.OpenTempDB(t)
	defer client.Close()

	first := client.Asset.Create().SetName("before.txt").SetContent([]byte("same")).SaveX(ctx)
	original, err := Create(ctx, client, []*ent.Asset{first})
	require.NoError(t, err)
	renamed := client.Asset.Create().SetName("after.txt").SetContent([]byte("same")).SaveX(ctx)
	byName, err := Create(ctx, client, []*ent.Asset{renamed})
	require.NoError(t, err)
	require.NotEqual(t, original.ID, byName.ID)

	client.Asset.UpdateOne(first).SetContent([]byte("changed")).ExecX(ctx)
	changed := client.Asset.GetX(ctx, first.ID)
	byContent, err := Create(ctx, client, []*ent.Asset{changed})
	require.NoError(t, err)
	require.NotEqual(t, original.ID, byContent.ID)
	require.NotEqual(t, byName.ID, byContent.ID)
	data, err := Read(byContent.Content, "before.txt")
	require.NoError(t, err)
	require.Equal(t, []byte("changed"), data)
}
