package auth_test

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent/enttest"
)

func TestContextFromTokens_Invalid(t *testing.T) {
	// Setup Dependencies
	var (
		driverName     = "sqlite3"
		dataSourceName = "file:ent?mode=memory&cache=shared&_fk=1"
	)
	graph := enttest.Open(t, driverName, dataSourceName, enttest.WithOptions())
	defer graph.Close()

	// Test ContextFromSessionToken with invalid token
	t.Run("ContextFromSessionToken_NotFound", func(t *testing.T) {
		// Pass a nil context first to check if it panics? No, Background is fine.
		ctx, err := auth.ContextFromSessionToken(context.Background(), graph, "invalid-token")
		require.Error(t, err)
		// Usually ent returns "ent: user not found"
		assert.Contains(t, err.Error(), "user not found")
		// The returned context should be nil if error?
		// Looking at context.go: if err != nil { return nil, err }
		assert.Nil(t, ctx)
	})

	// Test ContextFromAccessToken with invalid token
	t.Run("ContextFromAccessToken_NotFound", func(t *testing.T) {
		ctx, err := auth.ContextFromAccessToken(context.Background(), graph, "invalid-token")
		require.Error(t, err)
		assert.Contains(t, err.Error(), "user not found")
		assert.Nil(t, ctx)
	})
}

func TestContextHelpers_EdgeCases(t *testing.T) {
	ctx := context.Background()

	t.Run("EmptyContext", func(t *testing.T) {
		assert.Nil(t, auth.IdentityFromContext(ctx))
		assert.Nil(t, auth.UserFromContext(ctx))
		assert.False(t, auth.IsAuthenticatedContext(ctx))
		assert.False(t, auth.IsActivatedContext(ctx))
		assert.False(t, auth.IsAdminContext(ctx))
	})
}
