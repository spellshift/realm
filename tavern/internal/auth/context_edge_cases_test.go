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
	ctx := context.Background()

	// 1. Context with no identity
	assert.False(t, auth.IsAuthenticatedContext(ctx))
	assert.False(t, auth.IsActivatedContext(ctx))
	assert.False(t, auth.IsAdminContext(ctx))
	assert.Nil(t, auth.UserFromContext(ctx))
	assert.Nil(t, auth.IdentityFromContext(ctx))
}

func TestContextFromTokens_Invalid(t *testing.T) {
	ctx := context.Background()
	var (
		driverName     = "sqlite3"
		dataSourceName = "file:ent?mode=memory&cache=shared&_fk=1"
	)
	graph := enttest.Open(t, driverName, dataSourceName, enttest.WithOptions())
	defer graph.Close()

	// Test non-existent session token
	_, err := auth.ContextFromSessionToken(ctx, graph, "invalid_token")
	require.Error(t, err)

	// Test non-existent access token
	_, err = auth.ContextFromAccessToken(ctx, graph, "invalid_token")
	require.Error(t, err)
}
