package auth_test

import (
	"context"
	"testing"

	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/logging"
)

func TestContextFromSessionToken(t *testing.T) {
	// Setup Dependencies
	ctx := context.Background()

	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	// Test Data
	existingInactiveUser := graph.User.Create().
		SetName("test_inactive").
		SetOauthID("test_inactive").
		SetPhotoURL("http://google.com/").
		SetIsActivated(false).
		SetIsAdmin(false).
		SaveX(ctx)
	existingUser := graph.User.Create().
		SetName("test_user").
		SetOauthID("test_user").
		SetPhotoURL("http://google.com/").
		SetIsActivated(true).
		SetIsAdmin(false).
		SaveX(ctx)
	existingAdmin := graph.User.Create().
		SetName("test_admin").
		SetOauthID("test_admin").
		SetPhotoURL("http://google.com/").
		SetIsActivated(true).
		SetIsAdmin(true).
		SaveX(ctx)

	// Test Cases
	tests := []struct {
		name                string
		sessionToken        string
		wantIsAdmin         bool
		wantIsActivated     bool
		wantIsAuthenticated bool
		wantUserID          int
		wantNotFound        bool
	}{
		{
			name:                "InactiveUser",
			sessionToken:        existingInactiveUser.SessionToken,
			wantUserID:          existingInactiveUser.ID,
			wantIsAdmin:         false,
			wantIsActivated:     false,
			wantIsAuthenticated: true,
		},
		{
			name:                "User",
			sessionToken:        existingUser.SessionToken,
			wantUserID:          existingUser.ID,
			wantIsAdmin:         false,
			wantIsActivated:     true,
			wantIsAuthenticated: true,
		},
		{
			name:                "Admin",
			sessionToken:        existingAdmin.SessionToken,
			wantUserID:          existingAdmin.ID,
			wantIsAdmin:         true,
			wantIsActivated:     true,
			wantIsAuthenticated: true,
		},
		{
			name:         "NotFound",
			sessionToken: "invalid-token",
			wantNotFound: true,
		},
	}

	// Run Tests
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			authCtx, err := auth.ContextFromSessionToken(context.Background(), graph, tc.sessionToken)
			if tc.wantNotFound {
				require.Error(t, err)
				assert.True(t, ent.IsNotFound(err), "expected ent.NotFoundError, got: %v", err)
				assert.Nil(t, authCtx)
				return
			}
			require.NoError(t, err)

			assert.Equal(t, tc.wantIsActivated, auth.IsActivatedContext(authCtx))
			assert.Equal(t, tc.wantIsAdmin, auth.IsAdminContext(authCtx))
			assert.Equal(t, tc.wantIsAuthenticated, auth.IsAuthenticatedContext(authCtx))

			u := auth.UserFromContext(authCtx)
			if tc.wantUserID != 0 && u == nil {
				t.Fatalf("unexpected user id, want: %d, got: nil", tc.wantUserID)
			}
			assert.Equal(t, tc.wantUserID, u.ID)

			actor := logging.ActorFromContext(authCtx)
			assert.Equal(t, logging.ActorUser, actor.Kind)
			assert.Equal(t, u.Name, actor.Name)
			assert.Equal(t, tc.wantIsAdmin, actor.IsAdmin)
			assert.Equal(t, "cookie", actor.AuthMethod)
		})
	}

}

func TestContextFromAccessToken(t *testing.T) {
	// Setup Dependencies
	ctx := context.Background()

	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	// Test Data
	existingInactiveUser := graph.User.Create().
		SetName("test_inactive").
		SetOauthID("test_inactive").
		SetPhotoURL("http://google.com/").
		SetIsActivated(false).
		SetIsAdmin(false).
		SaveX(ctx)
	existingUser := graph.User.Create().
		SetName("test_user").
		SetOauthID("test_user").
		SetPhotoURL("http://google.com/").
		SetIsActivated(true).
		SetIsAdmin(false).
		SaveX(ctx)
	existingAdmin := graph.User.Create().
		SetName("test_admin").
		SetOauthID("test_admin").
		SetPhotoURL("http://google.com/").
		SetIsActivated(true).
		SetIsAdmin(true).
		SaveX(ctx)

	// Test Cases
	tests := []struct {
		name                string
		accessToken         string
		wantIsAdmin         bool
		wantIsActivated     bool
		wantIsAuthenticated bool
		wantUserID          int
		wantNotFound        bool
	}{
		{
			name:                "InactiveUser",
			accessToken:         existingInactiveUser.AccessToken,
			wantUserID:          existingInactiveUser.ID,
			wantIsAdmin:         false,
			wantIsActivated:     false,
			wantIsAuthenticated: true,
		},
		{
			name:                "User",
			accessToken:         existingUser.AccessToken,
			wantUserID:          existingUser.ID,
			wantIsAdmin:         false,
			wantIsActivated:     true,
			wantIsAuthenticated: true,
		},
		{
			name:                "Admin",
			accessToken:         existingAdmin.AccessToken,
			wantUserID:          existingAdmin.ID,
			wantIsAdmin:         true,
			wantIsActivated:     true,
			wantIsAuthenticated: true,
		},
		{
			name:         "NotFound",
			accessToken:  "invalid-token",
			wantNotFound: true,
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			authCtx, err := auth.ContextFromAccessToken(context.Background(), graph, tc.accessToken)
			if tc.wantNotFound {
				require.Error(t, err)
				assert.True(t, ent.IsNotFound(err), "expected ent.NotFoundError, got: %v", err)
				assert.Nil(t, authCtx)
				return
			}
			require.NoError(t, err)

			assert.Equal(t, tc.wantIsActivated, auth.IsActivatedContext(authCtx))
			assert.Equal(t, tc.wantIsAdmin, auth.IsAdminContext(authCtx))
			assert.Equal(t, tc.wantIsAuthenticated, auth.IsAuthenticatedContext(authCtx))

			u := auth.UserFromContext(authCtx)
			if tc.wantUserID != 0 && u == nil {
				t.Fatalf("unexpected user id, want: %d, got: nil", tc.wantUserID)
			}
			assert.Equal(t, tc.wantUserID, u.ID)

			actor := logging.ActorFromContext(authCtx)
			assert.Equal(t, logging.ActorUser, actor.Kind)
			assert.Equal(t, u.Name, actor.Name)
			assert.Equal(t, tc.wantIsAdmin, actor.IsAdmin)
			assert.Equal(t, "access_token", actor.AuthMethod)
		})
	}
}

func TestContextHelpers_EmptyContext(t *testing.T) {
	ctx := context.Background()

	assert.Nil(t, auth.IdentityFromContext(ctx))
	assert.Nil(t, auth.UserFromContext(ctx))
	assert.False(t, auth.IsAuthenticatedContext(ctx))
	assert.False(t, auth.IsActivatedContext(ctx))
	assert.False(t, auth.IsAdminContext(ctx))
}

func TestContextFromUserPreservesAuthMethod(t *testing.T) {
	ctx := context.Background()
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	u := graph.User.Create().
		SetName("operator").
		SetOauthID("op_oauth").
		SetPhotoURL("http://example.com/").
		SetIsActivated(true).
		SetIsAdmin(true).
		SaveX(ctx)

	base := logging.WithAuthMethod(ctx, "bypass")
	userCtx := auth.ContextFromUser(base, u)
	actor := logging.ActorFromContext(userCtx)
	assert.Equal(t, logging.ActorUser, actor.Kind)
	assert.Equal(t, "operator", actor.Name)
	assert.True(t, actor.IsAdmin)
	assert.Equal(t, "bypass", actor.AuthMethod)
}
