package auth_test

import (
	"context"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent"
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

	// Test Data
	existingUser := graph.User.Create().
		SetName("test_user_edge").
		SetOauthID("test_user_edge").
		SetPhotoURL("http://google.com/").
		SetIsActivated(true).
		SetIsAdmin(false).
		SaveX(ctx)

	t.Run("ContextFromSessionToken_NotFound", func(t *testing.T) {
		authCtx, err := auth.ContextFromSessionToken(context.Background(), graph, "invalid_token")
		require.Error(t, err)
		assert.True(t, ent.IsNotFound(err))
		assert.Nil(t, authCtx)
	})

	t.Run("ContextFromAccessToken_NotFound", func(t *testing.T) {
		authCtx, err := auth.ContextFromAccessToken(context.Background(), graph, "invalid_token")
		require.Error(t, err)
		assert.True(t, ent.IsNotFound(err))
		assert.Nil(t, authCtx)
	})

	t.Run("IdentityFromContext_Nil", func(t *testing.T) {
		id := auth.IdentityFromContext(context.Background())
		assert.Nil(t, id)
	})

	t.Run("UserFromContext_Nil", func(t *testing.T) {
		u := auth.UserFromContext(context.Background())
		assert.Nil(t, u)
	})

	t.Run("IsAuthenticatedContext_False", func(t *testing.T) {
		assert.False(t, auth.IsAuthenticatedContext(context.Background()))
	})

	t.Run("IsActivatedContext_False", func(t *testing.T) {
		assert.False(t, auth.IsActivatedContext(context.Background()))
	})

	t.Run("IsAdminContext_False", func(t *testing.T) {
		assert.False(t, auth.IsAdminContext(context.Background()))
	})

	t.Run("ValidUser_CheckMethods", func(t *testing.T) {
		authCtx, err := auth.ContextFromSessionToken(context.Background(), graph, existingUser.SessionToken)
		require.NoError(t, err)

		id := auth.IdentityFromContext(authCtx)
		require.NotNil(t, id)
		assert.Equal(t, "test_user_edge", id.String())
		assert.True(t, id.IsAuthenticated())
		assert.True(t, id.IsActivated())
		assert.False(t, id.IsAdmin())
	})
}
