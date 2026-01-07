package auth_test

import (
	"context"
	"testing"

	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent/enttest"
)

func TestContextEdgeCases(t *testing.T) {
	// Setup Dependencies
	ctx := context.Background()
	var (
		driverName     = "sqlite3"
		dataSourceName = "file:ent?mode=memory&cache=shared&_fk=1"
	)
	graph := enttest.Open(t, driverName, dataSourceName, enttest.WithOptions())
	defer graph.Close()

	// Test case: Invalid Session Token
	t.Run("InvalidSessionToken", func(t *testing.T) {
		authCtx, err := auth.ContextFromSessionToken(ctx, graph, "invalid_token")
		require.Error(t, err)
		assert.Nil(t, authCtx)
	})

	// Test case: Invalid Access Token
	t.Run("InvalidAccessToken", func(t *testing.T) {
		authCtx, err := auth.ContextFromAccessToken(ctx, graph, "invalid_token")
		require.Error(t, err)
		assert.Nil(t, authCtx)
	})

	// Test case: IdentityFromContext with no identity
	t.Run("IdentityFromContext_NoIdentity", func(t *testing.T) {
		id := auth.IdentityFromContext(ctx)
		assert.Nil(t, id)
	})

	// Test case: UserFromContext with no identity
	t.Run("UserFromContext_NoIdentity", func(t *testing.T) {
		u := auth.UserFromContext(ctx)
		assert.Nil(t, u)
	})

	// Test case: IsAuthenticatedContext with no identity
	t.Run("IsAuthenticatedContext_NoIdentity", func(t *testing.T) {
		authenticated := auth.IsAuthenticatedContext(ctx)
		assert.False(t, authenticated)
	})

	// Test case: IsActivatedContext with no identity
	t.Run("IsActivatedContext_NoIdentity", func(t *testing.T) {
		activated := auth.IsActivatedContext(ctx)
		assert.False(t, activated)
	})

	// Test case: IsAdminContext with no identity
	t.Run("IsAdminContext_NoIdentity", func(t *testing.T) {
		admin := auth.IsAdminContext(ctx)
		assert.False(t, admin)
	})
}
