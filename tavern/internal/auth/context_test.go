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

func TestContextFromSessionToken(t *testing.T) {
	// Setup Dependencies
	ctx := context.Background()
	var (
		driverName     = "sqlite3"
		dataSourceName = "file:ent?mode=memory&cache=shared&_fk=1"
	)
	graph := enttest.Open(t, driverName, dataSourceName, enttest.WithOptions())
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
		wantError           error
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
	}

	// Run Tests
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			authCtx, err := auth.ContextFromSessionToken(context.Background(), graph, tc.sessionToken)
			require.ErrorIs(t, err, tc.wantError)

			assert.Equal(t, tc.wantIsActivated, auth.IsActivatedContext(authCtx))
			assert.Equal(t, tc.wantIsAdmin, auth.IsAdminContext(authCtx))
			assert.Equal(t, tc.wantIsAuthenticated, auth.IsAuthenticatedContext(authCtx))

			u := auth.UserFromContext(authCtx)
			if tc.wantUserID != 0 && u == nil {
				t.Fatalf("unexpected user id, want: %d, got: nil", tc.wantUserID)
			}
			assert.Equal(t, tc.wantUserID, u.ID)
		})
	}

}

func TestContextFromAccessToken(t *testing.T) {
	// Setup Dependencies
	ctx := context.Background()
	var (
		driverName     = "sqlite3"
		dataSourceName = "file:ent?mode=memory&cache=shared&_fk=1"
	)
	graph := enttest.Open(t, driverName, dataSourceName, enttest.WithOptions())
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
		wantError           error
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
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			authCtx, err := auth.ContextFromAccessToken(context.Background(), graph, tc.accessToken)
			require.ErrorIs(t, err, tc.wantError)

			assert.Equal(t, tc.wantIsActivated, auth.IsActivatedContext(authCtx))
			assert.Equal(t, tc.wantIsAdmin, auth.IsAdminContext(authCtx))
			assert.Equal(t, tc.wantIsAuthenticated, auth.IsAuthenticatedContext(authCtx))

			u := auth.UserFromContext(authCtx)
			if tc.wantUserID != 0 && u == nil {
				t.Fatalf("unexpected user id, want: %d, got: nil", tc.wantUserID)
			}
			assert.Equal(t, tc.wantUserID, u.ID)
		})
	}

}
