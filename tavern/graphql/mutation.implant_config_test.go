package graphql_test

import (
	"context"
	"testing"

	"github.com/99designs/gqlgen/client"
	"github.com/99designs/gqlgen/graphql/handler"
	"github.com/kcarretto/realm/tavern/ent/enttest"
	"github.com/kcarretto/realm/tavern/ent/implantconfig"
	"github.com/kcarretto/realm/tavern/graphql"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	_ "github.com/mattn/go-sqlite3"
)

// TestCreateImplantConfig ensures the createImplantConfig mutation exhibits expected behavior.
func TestCreateImplantConfig(t *testing.T) {
	// Initialize Test Context
	ctx := context.Background()

	// Initialize DB Backend
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	// Create a new GraphQL client
	gqlClient := client.New(handler.NewDefaultServer(graphql.NewSchema(graph)))
	// Initialize sample data
	svcCfg1 := graph.ImplantServiceConfig.Create().
		SetName("meow-svc").
		SetExecutablePath("/bin/meow-svc").
		SaveX(ctx)
	svcCfg2 := graph.ImplantServiceConfig.Create().
		SetName("other-svc").
		SetExecutablePath("/bin/other-svc").
		SaveX(ctx)
	cbCfg1 := graph.ImplantCallbackConfig.Create().
		SetURI("https://existingconfig.realm.pub/").
		SaveX(ctx)
	cbCfg2 := graph.ImplantCallbackConfig.Create().
		SetURI("https://existingconfig.realm.pub/").
		SaveX(ctx)
	existingCfg := graph.ImplantConfig.Create().
		SetName("existing-config").
		AddCallbackConfigs(cbCfg1).
		AddServiceConfigs(svcCfg1).
		SaveX(ctx)

	// Run tests
	t.Run("New", newCreateImplantConfigTest(
		gqlClient,
		graphql.CreateImplantConfigInput{
			Name:              "new-cfg",
			ServiceConfigIDs:  []int{svcCfg1.ID, svcCfg2.ID},
			CallbackConfigIDs: []int{cbCfg1.ID, cbCfg2.ID},
		},
		func(t *testing.T, id int, err error) {
			require.NoError(t, err)
			assert.NotZero(t, id)

			cfg := graph.ImplantConfig.GetX(ctx, id)
			assert.Equal(t, "new-cfg", cfg.Name)
			assert.NotEmpty(t, cfg.AuthToken)

			svcCfgs, err := cfg.ServiceConfigs(ctx)
			require.NoError(t, err)
			assert.Len(t, svcCfgs, 2)
			for _, cfg := range svcCfgs {
				assert.Contains(t, []int{svcCfg1.ID, svcCfg2.ID}, cfg.ID)
			}

			cbCfgs, err := cfg.CallbackConfigs(ctx)
			require.NoError(t, err)
			assert.Len(t, cbCfgs, 2)
			for _, cfg := range cbCfgs {
				assert.Contains(t, []int{cbCfg1.ID, cbCfg2.ID}, cfg.ID)
			}
		},
	))
	t.Run("ExistingName", newCreateImplantConfigTest(
		gqlClient,
		graphql.CreateImplantConfigInput{
			Name: existingCfg.Name,
		},
		func(t *testing.T, id int, err error) {
			assert.ErrorContains(t, err, "ent: constraint failed: UNIQUE constraint failed: implant_configs.name")
			assert.Zero(t, id)
		},
	))
}

func newCreateImplantConfigTest(gqlClient *client.Client, input graphql.CreateImplantConfigInput, checks ...func(t *testing.T, id int, err error)) func(t *testing.T) {
	return func(t *testing.T) {
		// Define the mutatation for testing, taking the input as a variable
		mut := `mutation CreateImplantConfig($input: CreateImplantConfigInput!) { createImplantConfig(config:$input) { id } }`

		// Make our request to the GraphQL API
		var resp struct {
			CreateImplantConfig struct{ ID string }
		}
		err := gqlClient.Post(mut, &resp, client.Var("input", input))

		// Run checks with error (if any) and resulting ImplantConfig ID
		for _, check := range checks {
			check(t, convertID(resp.CreateImplantConfig.ID), err)
		}
	}
}

// TestUpdateImplantConfig ensures the updateImplantConfig mutation exhibits expected behavior.
func TestUpdateImplantConfig(t *testing.T) {
	// Initialize Test Context
	ctx := context.Background()

	// Initialize DB Backend
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	// Initialize sample data
	svcCfg1 := graph.ImplantServiceConfig.Create().
		SetName("meow-svc").
		SetExecutablePath("/bin/meow-svc").
		SaveX(ctx)
	svcCfg2 := graph.ImplantServiceConfig.Create().
		SetName("other-svc").
		SetExecutablePath("/bin/other-svc").
		SaveX(ctx)
	cbCfg1 := graph.ImplantCallbackConfig.Create().
		SetURI("https://existingconfig.realm.pub/").
		SaveX(ctx)
	cbCfg2 := graph.ImplantCallbackConfig.Create().
		SetURI("https://existingconfig.realm.pub/").
		SaveX(ctx)
	existingCfg := graph.ImplantConfig.Create().
		SetName("existing-config").
		AddCallbackConfigs(cbCfg1).
		AddServiceConfigs(svcCfg1).
		SaveX(ctx)

	// Create a new GraphQL client
	gqlClient := client.New(handler.NewDefaultServer(graphql.NewSchema(graph)))

	// Run tests
	t.Run("NoUpdate", newUpdateImplantConfigTest(
		gqlClient,
		graphql.UpdateImplantConfigInput{
			ID: existingCfg.ID,
		},
		func(t *testing.T, id int, err error) {
			require.NoError(t, err)
			assert.Equal(t, existingCfg.ID, id)

			cfg := graph.ImplantConfig.GetX(ctx, id)
			assert.Equal(t, existingCfg.Name, cfg.Name)
			assert.Equal(t, existingCfg.AuthToken, cfg.AuthToken)

			svcCfgs, err := cfg.ServiceConfigs(ctx)
			require.NoError(t, err)
			require.Len(t, svcCfgs, 1)
			assert.Equal(t, svcCfg1.ID, svcCfgs[0].ID)

			cbCfgs, err := cfg.CallbackConfigs(ctx)
			require.NoError(t, err)
			require.Len(t, cbCfgs, 1)
			assert.Equal(t, cbCfg1.ID, cbCfgs[0].ID)
		},
	))
	t.Run("ChangeConfigs", newUpdateImplantConfigTest(
		gqlClient,
		graphql.UpdateImplantConfigInput{
			ID:                      existingCfg.ID,
			AddServiceConfigIDs:     []int{svcCfg2.ID},
			RemoveServiceConfigIDs:  []int{svcCfg1.ID},
			AddCallbackConfigIDs:    []int{svcCfg2.ID},
			RemoveCallbackConfigIDs: []int{svcCfg1.ID},
		},
		func(t *testing.T, id int, err error) {
			require.NoError(t, err)
			assert.Equal(t, existingCfg.ID, id)

			cfg := graph.ImplantConfig.GetX(ctx, id)
			assert.Equal(t, existingCfg.Name, cfg.Name)
			assert.Equal(t, existingCfg.AuthToken, cfg.AuthToken)

			svcCfgs, err := cfg.ServiceConfigs(ctx)
			require.NoError(t, err)
			require.Len(t, svcCfgs, 1)
			assert.Equal(t, svcCfg2.ID, svcCfgs[0].ID)

			cbCfgs, err := cfg.CallbackConfigs(ctx)
			require.NoError(t, err)
			require.Len(t, cbCfgs, 1)
			assert.Equal(t, cbCfg2.ID, cbCfgs[0].ID)
		},
	))
	t.Run("AddExistingCallbackConfigs", newUpdateImplantConfigTest(
		gqlClient,
		graphql.UpdateImplantConfigInput{
			ID:                   existingCfg.ID,
			AddCallbackConfigIDs: []int{svcCfg1.ID, svcCfg2.ID},
		},
		func(t *testing.T, id int, err error) {
			assert.ErrorContains(t, err, "ent: constraint failed: add m2m edge for table implant_config_callbackConfigs: UNIQUE constraint failed: implant_config_callbackConfigs.implant_config_id")
			assert.Zero(t, id)
		},
	))
	t.Run("AddExistingServiceConfigs", newUpdateImplantConfigTest(
		gqlClient,
		graphql.UpdateImplantConfigInput{
			ID:                  existingCfg.ID,
			AddServiceConfigIDs: []int{svcCfg1.ID, svcCfg2.ID},
		},
		func(t *testing.T, id int, err error) {
			assert.ErrorContains(t, err, "ent: constraint failed: add m2m edge for table implant_config_serviceConfigs: UNIQUE constraint failed: implant_config_serviceConfigs.implant_config_id, implant_config_serviceConfigs.implant_service_config_id")
			assert.Zero(t, id)
		},
	))
	t.Run("AddNonExistingCallbackConfigs", newUpdateImplantConfigTest(
		gqlClient,
		graphql.UpdateImplantConfigInput{
			ID:                   existingCfg.ID,
			AddCallbackConfigIDs: []int{1337},
		},
		func(t *testing.T, id int, err error) {
			assert.ErrorContains(t, err, "ent: constraint failed: add m2m edge for table implant_config_callbackConfigs: FOREIGN KEY constraint failed")
			assert.Zero(t, id)
		},
	))
	t.Run("AddNonExistingServiceConfigs", newUpdateImplantConfigTest(
		gqlClient,
		graphql.UpdateImplantConfigInput{
			ID:                  existingCfg.ID,
			AddServiceConfigIDs: []int{1337},
		},
		func(t *testing.T, id int, err error) {
			assert.ErrorContains(t, err, "ent: constraint failed: add m2m edge for table implant_config_serviceConfigs: FOREIGN KEY constraint failed")
			assert.Zero(t, id)
		},
	))
	t.Run("NotExists", newUpdateImplantConfigTest(
		gqlClient,
		graphql.UpdateImplantConfigInput{
			ID: -1,
		},
		func(t *testing.T, id int, err error) {
			assert.ErrorContains(t, err, "ent: implant_config not found")
			assert.Zero(t, id)
		},
	))
}

func newUpdateImplantConfigTest(gqlClient *client.Client, input graphql.UpdateImplantConfigInput, checks ...func(t *testing.T, id int, err error)) func(t *testing.T) {
	return func(t *testing.T) {
		// Define the mutatation for testing, taking the input as a variable
		mut := `mutation UpdateImplantConfig($input: UpdateImplantConfigInput!) { updateImplantConfig(config:$input) { id } }`

		// Make our request to the GraphQL API
		var resp struct {
			UpdateImplantConfig struct{ ID string }
		}
		err := gqlClient.Post(mut, &resp, client.Var("input", input))

		// Run checks with error (if any) and resulting ImplantConfig ID
		for _, check := range checks {
			check(t, convertID(resp.UpdateImplantConfig.ID), err)
		}
	}
}

// TestDeleteImplantConfig ensures the deleteImplantConfig mutation exhibits expected behavior.
func TestDeleteImplantConfig(t *testing.T) {
	// Initialize Test Context
	ctx := context.Background()

	// Initialize DB Backend
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	// Initialize sample data
	existingConfig := graph.ImplantConfig.Create().
		SetName("delete-me").
		SaveX(ctx)

	// Create a new GraphQL client
	gqlClient := client.New(handler.NewDefaultServer(graphql.NewSchema(graph)))

	// Run tests
	t.Run("Existing", newDeleteImplantConfigTest(
		gqlClient,
		existingConfig.ID,
		func(t *testing.T, id int, err error) {
			require.NoError(t, err)
			assert.NotZero(t, id)
			assert.Equal(t, existingConfig.ID, id)

			exists := graph.ImplantConfig.Query().
				Where(implantconfig.ID(id)).
				ExistX(ctx)
			assert.False(t, exists)
		},
	))
	t.Run("NotExists", newDeleteImplantConfigTest(
		gqlClient,
		-1,
		func(t *testing.T, id int, err error) {
			assert.Zero(t, id)
			assert.ErrorContains(t, err, "ent: implant_config not found")
		},
	))
}

func newDeleteImplantConfigTest(gqlClient *client.Client, id int, checks ...func(t *testing.T, id int, err error)) func(t *testing.T) {
	return func(t *testing.T) {
		// Define the mutatation for testing, taking the input as a variable
		mut := `mutation DeleteImplantConfig($input: ID!) { deleteImplantConfig(id:$input) }`

		// Make our request to the GraphQL API
		var resp struct {
			DeleteImplantConfig string
		}
		err := gqlClient.Post(mut, &resp, client.Var("input", id))

		// Run checks with error (if any) and resulting ImplantConfig ID
		for _, check := range checks {
			check(t, convertID(resp.DeleteImplantConfig), err)
		}
	}
}
