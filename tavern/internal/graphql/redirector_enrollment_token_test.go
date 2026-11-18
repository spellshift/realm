package graphql_test

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"net/http"
	"testing"
	"time"

	"github.com/99designs/gqlgen/client"
	"github.com/99designs/gqlgen/graphql/handler"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/auth"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/graphql"
	tavernhttp "realm.pub/tavern/internal/http"
)

func TestCreateRedirectorEnrollmentTokenStoresOnlyHash(t *testing.T) {
	graph := enttest.OpenTempDB(t)
	defer graph.Close()
	ctx := context.Background()
	_, err := graph.User.Create().SetOauthID("admin").SetPhotoURL("https://example.com/photo").SetName("Admin").SetSessionToken("redirector-test-session").SetAccessToken("access").SetIsActivated(true).SetIsAdmin(true).Save(ctx)
	require.NoError(t, err)
	srv := tavernhttp.NewServer(tavernhttp.RouteMap{
		"/graphql": handler.NewDefaultServer(graphql.NewSchema(graph, importerFake{graph})),
	}, tavernhttp.WithAuthentication(graph))
	gqlClient := client.New(srv, client.Path("/graphql"))
	var response struct {
		CreateRedirectorEnrollmentToken string `json:"createRedirectorEnrollmentToken"`
	}
	err = gqlClient.Post(`mutation { createRedirectorEnrollmentToken }`, &response, client.AddCookie(&http.Cookie{Name: auth.SessionCookieName, Value: "redirector-test-session", Expires: time.Now().Add(time.Hour)}))
	require.NoError(t, err)
	require.NotEmpty(t, response.CreateRedirectorEnrollmentToken)
	records, err := graph.RedirectorEnrollmentToken.Query().All(ctx)
	require.NoError(t, err)
	require.Len(t, records, 1)
	hash := sha256.Sum256([]byte(response.CreateRedirectorEnrollmentToken))
	require.Equal(t, hex.EncodeToString(hash[:]), records[0].TokenHash)
	require.False(t, records[0].Used)
}
