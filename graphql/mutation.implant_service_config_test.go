package graphql_test

import (
	"context"
	"testing"

	"github.com/99designs/gqlgen/client"
	"github.com/99designs/gqlgen/graphql/handler"
	"github.com/kcarretto/realm/ent/enttest"
	"github.com/kcarretto/realm/ent/implantserviceconfig"
	"github.com/kcarretto/realm/graphql"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	_ "github.com/mattn/go-sqlite3"
)

// TestCreateServiceServiceConfig ensures the createImplantServiceConfig mutation exhibits expected behavior.
func TestCreateImplantServiceConfig(t *testing.T) {
	// Initialize Test Context
	ctx := context.Background()

	// Initialize DB Backend
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	// Initialize sample data
	existingConfig := graph.ImplantServiceConfig.Create().
		SetName("meow-svc").
		SetExecutablePath("/bin/meow-svc").
		SaveX(ctx)

	// Create a new GraphQL client
	gqlClient := client.New(handler.NewDefaultServer(graphql.NewSchema(graph)))

	// Run tests
	t.Run("WithDefaults", newCreateImplantServiceConfigTest(
		gqlClient,
		graphql.CreateImplantServiceConfigInput{
			Name:           "test-service",
			ExecutablePath: "/bin/test-service",
		},
		func(t *testing.T, id int, err error) {
			require.NoError(t, err)
			assert.NotZero(t, id)

			cfg := graph.ImplantServiceConfig.GetX(ctx, id)
			assert.Equal(t, "test-service", cfg.Name)
			assert.NotEmpty(t, cfg.Description)
			assert.Equal(t, "/bin/test-service", cfg.ExecutablePath)
		},
	))
	var (
		expectedName           string = "new-svc"
		expectedDescription           = "Some Info Here"
		expectedExecutablePath        = `C:\new-svc`
	)
	t.Run("WithValues", newCreateImplantServiceConfigTest(
		gqlClient,
		graphql.CreateImplantServiceConfigInput{
			Name:           expectedName,
			Description:    &expectedDescription,
			ExecutablePath: expectedExecutablePath,
		},
		func(t *testing.T, id int, err error) {
			require.NoError(t, err)
			assert.NotZero(t, id)

			cfg := graph.ImplantServiceConfig.GetX(ctx, id)
			assert.Equal(t, expectedName, cfg.Name)
			assert.Equal(t, expectedDescription, cfg.Description)
			assert.Equal(t, expectedExecutablePath, cfg.ExecutablePath)
		},
	))
	t.Run("Duplicate", newCreateImplantServiceConfigTest(
		gqlClient,
		graphql.CreateImplantServiceConfigInput{
			Name:           existingConfig.Name,
			Description:    &expectedDescription,
			ExecutablePath: expectedExecutablePath,
		},
		func(t *testing.T, id int, err error) {
			assert.Zero(t, id)
			assert.ErrorContains(t, err, "ent: constraint failed: UNIQUE constraint failed: implant_service_configs.name")
		},
	))
}

func newCreateImplantServiceConfigTest(gqlClient *client.Client, input graphql.CreateImplantServiceConfigInput, checks ...func(t *testing.T, id int, err error)) func(t *testing.T) {
	return func(t *testing.T) {
		// Define the mutatation for testing, taking the input as a variable
		mut := `mutation CreateImplantServiceConfig($input: CreateImplantServiceConfigInput!) { createImplantServiceConfig(config:$input) { id } }`

		// Make our request to the GraphQL API
		var resp struct {
			CreateImplantServiceConfig struct{ ID int }
		}
		err := gqlClient.Post(mut, &resp, client.Var("input", input))

		// Run checks with error (if any) and resulting ImplantServiceConfig ID
		for _, check := range checks {
			check(t, resp.CreateImplantServiceConfig.ID, err)
		}
	}
}

// TestUpdateImplantServiceConfig ensures the updateImplantServiceConfig mutation exhibits expected behavior.
func TestUpdateImplantServiceConfig(t *testing.T) {
	// Initialize Test Context
	ctx := context.Background()

	// Initialize DB Backend
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	// Initialize sample data
	existingConfig := graph.ImplantServiceConfig.Create().
		SetName("meow-svc").
		SetExecutablePath("/bin/meow-svc").
		SaveX(ctx)
	existingConfig2 := graph.ImplantServiceConfig.Create().
		SetName("other-svc").
		SetExecutablePath("/bin/other-svc").
		SaveX(ctx)

	// Create a new GraphQL client
	gqlClient := client.New(handler.NewDefaultServer(graphql.NewSchema(graph)))

	// Run tests
	t.Run("NoUpdate", newUpdateImplantServiceConfigTest(
		gqlClient,
		graphql.UpdateImplantServiceConfigInput{
			ID: existingConfig.ID,
		},
		func(t *testing.T, id int, err error) {
			require.NoError(t, err)
			assert.NotZero(t, id)
			assert.Equal(t, existingConfig.ID, id)

			cfg := graph.ImplantServiceConfig.GetX(ctx, id)
			assert.Equal(t, existingConfig.Name, cfg.Name)
			assert.Equal(t, existingConfig.Description, cfg.Description)
			assert.Equal(t, existingConfig.ExecutablePath, cfg.ExecutablePath)
		},
	))
	var (
		expectedName           string = "meow-svc"
		expectedDescription           = "Some Info Here"
		expectedExecutablePath        = `C:\meow-svc`
	)
	t.Run("UpdateFields", newUpdateImplantServiceConfigTest(
		gqlClient,
		graphql.UpdateImplantServiceConfigInput{
			ID:             existingConfig.ID,
			Name:           &expectedName,
			Description:    &expectedDescription,
			ExecutablePath: &expectedExecutablePath,
		},
		func(t *testing.T, id int, err error) {
			require.NoError(t, err)
			assert.NotZero(t, id)
			assert.Equal(t, existingConfig.ID, id)

			cfg := graph.ImplantServiceConfig.GetX(ctx, id)
			assert.Equal(t, expectedName, cfg.Name)
			assert.Equal(t, expectedDescription, cfg.Description)
			assert.Equal(t, expectedExecutablePath, cfg.ExecutablePath)
		},
	))
	t.Run("NotExists", newUpdateImplantServiceConfigTest(
		gqlClient,
		graphql.UpdateImplantServiceConfigInput{
			ID: -1,
		},
		func(t *testing.T, id int, err error) {
			assert.Zero(t, id)
			assert.ErrorContains(t, err, "ent: implant_service_config not found")
		},
	))
	t.Run("Duplicate", newUpdateImplantServiceConfigTest(
		gqlClient,
		graphql.UpdateImplantServiceConfigInput{
			ID:             existingConfig.ID,
			Name:           &existingConfig2.Name,
			Description:    &expectedDescription,
			ExecutablePath: &expectedExecutablePath,
		},
		func(t *testing.T, id int, err error) {
			assert.Zero(t, id)
			assert.ErrorContains(t, err, "ent: constraint failed: UNIQUE constraint failed: implant_service_configs.name")
		},
	))
}

func newUpdateImplantServiceConfigTest(gqlClient *client.Client, input graphql.UpdateImplantServiceConfigInput, checks ...func(t *testing.T, id int, err error)) func(t *testing.T) {
	return func(t *testing.T) {
		// Define the mutatation for testing, taking the input as a variable
		mut := `mutation UpdateImplantServiceConfig($input: UpdateImplantServiceConfigInput!) { updateImplantServiceConfig(config:$input) { id } }`

		// Make our request to the GraphQL API
		var resp struct {
			UpdateImplantServiceConfig struct{ ID int }
		}
		err := gqlClient.Post(mut, &resp, client.Var("input", input))

		// Run checks with error (if any) and resulting ImplantServiceConfig ID
		for _, check := range checks {
			check(t, resp.UpdateImplantServiceConfig.ID, err)
		}
	}
}

// TestDeleteImplantServiceConfig ensures the deleteImplantServiceConfig mutation exhibits expected behavior.
func TestDeleteImplantServiceConfig(t *testing.T) {
	// Initialize Test Context
	ctx := context.Background()

	// Initialize DB Backend
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	// Initialize sample data
	existingConfig := graph.ImplantServiceConfig.Create().
		SetName("meow-svc").
		SetExecutablePath("/bin/meow-svc").
		SaveX(ctx)

	// Create a new GraphQL client
	gqlClient := client.New(handler.NewDefaultServer(graphql.NewSchema(graph)))

	// Run tests
	t.Run("Existing", newDeleteImplantServiceConfigTest(
		gqlClient,
		existingConfig.ID,
		func(t *testing.T, id int, err error) {
			require.NoError(t, err)
			assert.NotZero(t, id)
			assert.Equal(t, existingConfig.ID, id)

			exists := graph.ImplantServiceConfig.Query().
				Where(implantserviceconfig.ID(id)).
				ExistX(ctx)
			assert.False(t, exists)
		},
	))
	t.Run("NotExists", newDeleteImplantServiceConfigTest(
		gqlClient,
		-1,
		func(t *testing.T, id int, err error) {
			assert.Zero(t, id)
			assert.ErrorContains(t, err, "ent: implant_service_config not found")
		},
	))
}

func newDeleteImplantServiceConfigTest(gqlClient *client.Client, id int, checks ...func(t *testing.T, id int, err error)) func(t *testing.T) {
	return func(t *testing.T) {
		// Define the mutatation for testing, taking the input as a variable
		mut := `mutation DeleteImplantServiceConfig($input: ID!) { deleteImplantServiceConfig(id:$input) }`

		// Make our request to the GraphQL API
		var resp struct {
			DeleteImplantServiceConfig int
		}
		err := gqlClient.Post(mut, &resp, client.Var("input", id))

		// Run checks with error (if any) and resulting ImplantServiceConfig ID
		for _, check := range checks {
			check(t, resp.DeleteImplantServiceConfig, err)
		}
	}
}
