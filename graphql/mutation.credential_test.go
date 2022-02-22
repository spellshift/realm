package graphql_test

import (
	"context"
	"testing"

	"github.com/99designs/gqlgen/client"
	"github.com/99designs/gqlgen/graphql/handler"
	"github.com/kcarretto/realm/ent/credential"
	"github.com/kcarretto/realm/ent/enttest"
	"github.com/kcarretto/realm/graphql"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	_ "github.com/mattn/go-sqlite3"
)

// TestCreateCredential ensures the createCredential mutation exhibits expected behavior.
func TestCreateCredential(t *testing.T) {
	// Initialize Test Context
	ctx := context.Background()

	// Initialize DB Backend
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	// Initialize sample data
	target := graph.Target.Create().
		SetName("Target1").
		SetForwardConnectIP("10.0.0.1").
		SaveX(ctx)

	// Create a new GraphQL client
	gqlClient := client.New(handler.NewDefaultServer(graphql.NewSchema(graph)))

	// Run tests
	t.Run("New", newCreateCredentialTest(
		gqlClient,
		graphql.CreateCredentialInput{
			Principal: "root",
			Secret:    "changeme",
			Kind:      credential.KindPassword,
			TargetID:  target.ID,
		},
		func(t *testing.T, id int, err error) {
			require.NoError(t, err)
			assert.NotZero(t, id)

			cred := graph.Credential.GetX(ctx, id)
			assert.Equal(t, "root", cred.Principal)
			assert.Equal(t, "changeme", cred.Secret)
			assert.Equal(t, credential.KindPassword, cred.Kind)

			credTarget, edgeErr := cred.Target(ctx)
			require.NoError(t, edgeErr)
			require.NotNil(t, credTarget)
			assert.Equal(t, target.ID, credTarget.ID)
		},
	))

	t.Run("NoTarget", newCreateCredentialTest(
		gqlClient,
		graphql.CreateCredentialInput{
			Principal: "root",
			Secret:    "changeme",
			Kind:      credential.KindPassword,
		},
		func(t *testing.T, id int, err error) {
			assert.ErrorContains(t, err, "ent: constraint failed: FOREIGN KEY constraint failed")
			assert.Zero(t, id)
		},
	))
}

func newCreateCredentialTest(gqlClient *client.Client, input graphql.CreateCredentialInput, checks ...func(t *testing.T, id int, err error)) func(t *testing.T) {
	return func(t *testing.T) {
		// Define the mutatation for testing, taking the input as a variable
		mut := `mutation CreateCredential($input: CreateCredentialInput!) { createCredential(credential:$input) { id } }`

		// Make our request to the GraphQL API
		var resp struct {
			CreateCredential struct{ ID int }
		}
		err := gqlClient.Post(mut, &resp, client.Var("input", input))

		// Run checks with error (if any) and resulting CredentialID
		for _, check := range checks {
			check(t, resp.CreateCredential.ID, err)
		}
	}
}
