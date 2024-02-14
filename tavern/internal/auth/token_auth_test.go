package auth_test

import (
	"context"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent/enttest"
)

func TestNewTokenRedirectHandler(t *testing.T) {
	// Setup Dependencies
	ctx := context.Background()
	var (
		driverName     = "sqlite3"
		dataSourceName = "file:ent?mode=memory&cache=shared&_fk=1"
	)
	graph := enttest.Open(t, driverName, dataSourceName, enttest.WithOptions())
	defer graph.Close()
	handler := auth.NewTokenRedirectHandler()

	// Test Data
	existingAdmin := graph.User.Create().
		SetName("test_admin").
		SetOauthID("test_admin").
		SetPhotoURL("http://google.com/").
		SetIsActivated(true).
		SetIsAdmin(true).
		SaveX(ctx)
	adminCtx, err := auth.ContextFromSessionToken(ctx, graph, existingAdmin.SessionToken)
	require.NoError(t, err)

	// Test Cases
	tests := []struct {
		name string
		req  *http.Request

		wantLocationPrefix string
		wantCode           int
	}{
		{
			name: "Redirect",
			req: httptest.NewRequest(
				http.MethodGet,
				"/access_token/redirect?redir_port=8081",
				nil,
			).WithContext(adminCtx),
			wantLocationPrefix: "http://127.0.0.1:8081?access_token=",
			wantCode:           http.StatusFound,
		},
		{
			name: "Unauthenticated",
			req: httptest.NewRequest(
				http.MethodGet,
				"/access_token/redirect?redir_port=8081",
				nil,
			),
			wantCode: http.StatusUnauthorized,
		},
		{
			name: "InvalidPortRange",
			req: httptest.NewRequest(
				http.MethodGet,
				"/access_token/redirect?redir_port=13371337",
				nil,
			).WithContext(adminCtx),
			wantCode: http.StatusBadRequest,
		},
		{
			name: "InvalidPortType",
			req: httptest.NewRequest(
				http.MethodGet,
				"/access_token/redirect?redir_port=abcdefg",
				nil,
			).WithContext(adminCtx),
			wantCode: http.StatusBadRequest,
		},
		{
			name: "NoPort",
			req: httptest.NewRequest(
				http.MethodGet,
				"/access_token/redirect",
				nil,
			).WithContext(adminCtx),
			wantCode: http.StatusBadRequest,
		},
	}

	// Run Tests
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			w := httptest.NewRecorder()
			handler.ServeHTTP(w, tc.req)
			resp := w.Result()
			assert.Equal(t, tc.wantCode, resp.StatusCode)
			assert.True(t, strings.HasPrefix(resp.Header.Get("Location"), tc.wantLocationPrefix))
		})
	}
}
