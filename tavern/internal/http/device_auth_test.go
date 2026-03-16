package http_test

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent/deviceauth"
	"realm.pub/tavern/internal/ent/enttest"
	tavernhttp "realm.pub/tavern/internal/http"
)

func TestRDARevokeHandler(t *testing.T) {
	ctx := context.Background()
	var (
		driverName     = "sqlite3"
		dataSourceName = "file:ent?mode=memory&cache=shared&_fk=1"
	)
	graph := enttest.Open(t, driverName, dataSourceName, enttest.WithOptions())
	defer graph.Close()

	// Set up users
	adminUser := graph.User.Create().
		SetName("admin").
		SetOauthID("admin").
		SetPhotoURL("http://google.com/").
		SetIsActivated(true).
		SetIsAdmin(true).
		SaveX(ctx)

	normalUser := graph.User.Create().
		SetName("user").
		SetOauthID("user").
		SetPhotoURL("http://google.com/").
		SetIsActivated(true).
		SetIsAdmin(false).
		SaveX(ctx)

	otherUser := graph.User.Create().
		SetName("other").
		SetOauthID("other").
		SetPhotoURL("http://google.com/").
		SetIsActivated(true).
		SetIsAdmin(false).
		SaveX(ctx)

	// Set up device auths
	daAdmin := graph.DeviceAuth.Create().
		SetUserCode("code_admin").
		SetDeviceCode("device_admin").
		SetStatus(deviceauth.StatusAPPROVED).
		SetExpiresAt(time.Now().Add(1 * time.Hour)).
		SetUser(adminUser).
		SaveX(ctx)

	daNormal := graph.DeviceAuth.Create().
		SetUserCode("code_normal").
		SetDeviceCode("device_normal").
		SetStatus(deviceauth.StatusAPPROVED).
		SetExpiresAt(time.Now().Add(1 * time.Hour)).
		SetUser(normalUser).
		SaveX(ctx)

	daOther := graph.DeviceAuth.Create().
		SetUserCode("code_other").
		SetDeviceCode("device_other").
		SetStatus(deviceauth.StatusAPPROVED).
		SetExpiresAt(time.Now().Add(1 * time.Hour)).
		SetUser(otherUser).
		SaveX(ctx)

	handler := tavernhttp.NewRDARevokeHandler(graph)

	tests := []struct {
		name       string
		method     string
		body       map[string]interface{}
		userToken  string
		wantStatus int
	}{
		{
			name:       "Wrong method",
			method:     http.MethodGet,
			body:       nil,
			userToken:  "",
			wantStatus: http.StatusMethodNotAllowed,
		},
		{
			name:       "Invalid body",
			method:     http.MethodPost,
			body:       nil,
			userToken:  "",
			wantStatus: http.StatusBadRequest,
		},
		{
			name:       "Unauthenticated",
			method:     http.MethodPost,
			body:       map[string]interface{}{"user_code": daNormal.UserCode},
			userToken:  "",
			wantStatus: http.StatusUnauthorized,
		},
		{
			name:       "Invalid user code",
			method:     http.MethodPost,
			body:       map[string]interface{}{"user_code": "invalid"},
			userToken:  normalUser.SessionToken,
			wantStatus: http.StatusNotFound,
		},
		{
			name:       "Revoke own device",
			method:     http.MethodPost,
			body:       map[string]interface{}{"user_code": daNormal.UserCode},
			userToken:  normalUser.SessionToken,
			wantStatus: http.StatusOK,
		},
		{
			name:       "Revoke other user device (non-admin)",
			method:     http.MethodPost,
			body:       map[string]interface{}{"user_code": daOther.UserCode},
			userToken:  normalUser.SessionToken,
			wantStatus: http.StatusUnauthorized,
		},
		{
			name:       "Revoke other user device (admin)",
			method:     http.MethodPost,
			body:       map[string]interface{}{"user_code": daOther.UserCode},
			userToken:  adminUser.SessionToken,
			wantStatus: http.StatusOK,
		},
		{
			name:       "Revoke own device (admin)",
			method:     http.MethodPost,
			body:       map[string]interface{}{"user_code": daAdmin.UserCode},
			userToken:  adminUser.SessionToken,
			wantStatus: http.StatusOK,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var bodyBytes []byte
			if tt.body != nil {
				bodyBytes, _ = json.Marshal(tt.body)
			} else if tt.method == http.MethodPost {
				bodyBytes = []byte("invalid json")
			}
			req := httptest.NewRequest(tt.method, "/auth/rda/revoke", bytes.NewBuffer(bodyBytes))

			if tt.userToken != "" {
				authCtx, _ := auth.ContextFromSessionToken(req.Context(), graph, tt.userToken)
				req = req.WithContext(authCtx)
			}

			w := httptest.NewRecorder()
			handler.ServeHTTP(w, req)

			assert.Equal(t, tt.wantStatus, w.Code)
		})
	}
}
