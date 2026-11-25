package enrollment_test

import (
	"context"
	"testing"

	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/enrollment"
	"realm.pub/tavern/internal/ent/enttest"
)

func TestTokensAreRoleScopedAndSingleUse(t *testing.T) {
	ctx := context.Background()
	client := enttest.OpenTempDB(t)
	defer client.Close()
	for _, kind := range []enrollment.Kind{enrollment.Builder, enrollment.Redirector} {
		token, _, err := enrollment.Issue(ctx, client, kind)
		require.NoError(t, err)
		other := enrollment.Builder
		if kind == enrollment.Builder {
			other = enrollment.Redirector
		}
		available, err := enrollment.Available(ctx, client, other, token)
		require.NoError(t, err)
		require.False(t, available)
		available, err = enrollment.Available(ctx, client, kind, token)
		require.NoError(t, err)
		require.True(t, available)
		tx, err := client.Tx(ctx)
		require.NoError(t, err)
		consumed, err := enrollment.Consume(ctx, tx, kind, token)
		require.NoError(t, err)
		require.True(t, consumed)
		require.NoError(t, tx.Commit())
		available, err = enrollment.Available(ctx, client, kind, token)
		require.NoError(t, err)
		require.False(t, available)
		tx, err = client.Tx(ctx)
		require.NoError(t, err)
		consumed, err = enrollment.Consume(ctx, tx, kind, token)
		require.NoError(t, err)
		require.False(t, consumed)
		require.NoError(t, tx.Rollback())
	}
}
