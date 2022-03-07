package graphql_test

import (
	"context"
	"testing"

	"github.com/99designs/gqlgen/client"
	"github.com/99designs/gqlgen/graphql/handler"
	"github.com/kcarretto/realm/tavern/ent/deploymentconfig"
	"github.com/kcarretto/realm/tavern/ent/enttest"
	"github.com/kcarretto/realm/tavern/graphql"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	_ "github.com/mattn/go-sqlite3"
)

// TestCreateDeploymentConfig ensures the createDeploymentConfig mutation exhibits expected behavior.
func TestCreateDeploymentConfig(t *testing.T) {
	// Initialize Test Context
	ctx := context.Background()

	// Initialize DB Backend
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	// Initialize sample data
	implantConfig := graph.ImplantConfig.Create().
		SetName("test-implant").
		SaveX(ctx)
	f := graph.File.Create().
		SetName("test-implant").
		SetContent([]byte("uh oh")).
		SetHash("ABCDEFG").
		SaveX(ctx)
	existingConfig := graph.DeploymentConfig.Create().
		SetName("test-deployment").
		SaveX(ctx)

	// Create a new GraphQL client
	gqlClient := client.New(handler.NewDefaultServer(graphql.NewSchema(graph)))

	// Run tests
	var (
		expectedCmd      = "/bin/sh -c '/dst'"
		expectedStartCmd = true
		expectedDst      = "/dst"
	)
	t.Run("New", newCreateDeploymentConfigTest(
		gqlClient,
		graphql.CreateDeploymentConfigInput{
			Name:            "new-deployment",
			Cmd:             &expectedCmd,
			StartCmd:        &expectedStartCmd,
			FileDst:         &expectedDst,
			ImplantConfigID: &implantConfig.ID,
			FileID:          &f.ID,
		},
		func(t *testing.T, id int, err error) {
			require.NoError(t, err)
			assert.NotZero(t, id)

			cfg := graph.DeploymentConfig.GetX(ctx, id)
			assert.Equal(t, "new-deployment", cfg.Name)
			assert.Equal(t, expectedCmd, cfg.Cmd)
			assert.Equal(t, expectedDst, cfg.FileDst)

			impCfg, err := cfg.ImplantConfig(ctx)
			require.NoError(t, err)
			assert.Equal(t, implantConfig.ID, impCfg.ID)

			cfgFile, err := cfg.File(ctx)
			require.NoError(t, err)
			assert.Equal(t, f.ID, cfgFile.ID)
		},
	))
	t.Run("Duplicate", newCreateDeploymentConfigTest(
		gqlClient,
		graphql.CreateDeploymentConfigInput{
			Name: existingConfig.Name,
		},
		func(t *testing.T, id int, err error) {
			assert.Zero(t, id)
			assert.ErrorContains(t, err, "ent: constraint failed: UNIQUE constraint failed: deployment_configs.name")
		},
	))
}

func newCreateDeploymentConfigTest(gqlClient *client.Client, input graphql.CreateDeploymentConfigInput, checks ...func(t *testing.T, id int, err error)) func(t *testing.T) {
	return func(t *testing.T) {
		// Define the mutatation for testing, taking the input as a variable
		mut := `mutation CreateDeploymentConfig($input: CreateDeploymentConfigInput!) { createDeploymentConfig(config:$input) { id } }`

		// Make our request to the GraphQL API
		var resp struct {
			CreateDeploymentConfig struct{ ID string }
		}
		err := gqlClient.Post(mut, &resp, client.Var("input", input))

		// Run checks with error (if any) and resulting DeploymentConfig ID
		for _, check := range checks {
			check(t, convertID(resp.CreateDeploymentConfig.ID), err)
		}
	}
}

// TestUpdateDeploymentConfig ensures the updateDeploymentConfig mutation exhibits expected behavior.
func TestUpdateDeploymentConfig(t *testing.T) {
	// Initialize Test Context
	ctx := context.Background()

	// Initialize DB Backend
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	// Initialize sample data
	existingConfig := graph.DeploymentConfig.Create().
		SetName("meow-deploy").
		SaveX(ctx)
	existingConfig2 := graph.DeploymentConfig.Create().
		SetName("other-deploy").
		SaveX(ctx)

	// Create a new GraphQL client
	gqlClient := client.New(handler.NewDefaultServer(graphql.NewSchema(graph)))

	// Run tests
	t.Run("NoUpdate", newUpdateDeploymentConfigTest(
		gqlClient,
		graphql.UpdateDeploymentConfigInput{
			ID: existingConfig.ID,
		},
		func(t *testing.T, id int, err error) {
			require.NoError(t, err)
			assert.NotZero(t, id)
			assert.Equal(t, existingConfig.ID, id)

			cfg := graph.DeploymentConfig.GetX(ctx, id)
			assert.Equal(t, existingConfig.Name, cfg.Name)
			assert.Equal(t, existingConfig.Cmd, cfg.Cmd)
			assert.Equal(t, existingConfig.FileDst, cfg.FileDst)
		},
	))
	var (
		expectedName    string = "meow-deploy"
		expectedCmd            = "echo hi"
		expectedFileDst        = `C:\meow-svc`
	)
	t.Run("UpdateFields", newUpdateDeploymentConfigTest(
		gqlClient,
		graphql.UpdateDeploymentConfigInput{
			ID:      existingConfig.ID,
			Name:    &expectedName,
			Cmd:     &expectedCmd,
			FileDst: &expectedFileDst,
		},
		func(t *testing.T, id int, err error) {
			require.NoError(t, err)
			assert.NotZero(t, id)
			assert.Equal(t, existingConfig.ID, id)

			cfg := graph.DeploymentConfig.GetX(ctx, id)
			assert.Equal(t, expectedName, cfg.Name)
			assert.Equal(t, expectedCmd, cfg.Cmd)
			assert.Equal(t, expectedFileDst, cfg.FileDst)
		},
	))
	t.Run("NotExists", newUpdateDeploymentConfigTest(
		gqlClient,
		graphql.UpdateDeploymentConfigInput{
			ID: -1,
		},
		func(t *testing.T, id int, err error) {
			assert.Zero(t, id)
			assert.ErrorContains(t, err, "ent: deployment_config not found")
		},
	))
	t.Run("Duplicate", newUpdateDeploymentConfigTest(
		gqlClient,
		graphql.UpdateDeploymentConfigInput{
			ID:   existingConfig.ID,
			Name: &existingConfig2.Name,
		},
		func(t *testing.T, id int, err error) {
			assert.Zero(t, id)
			assert.ErrorContains(t, err, "ent: constraint failed: UNIQUE constraint failed: deployment_configs.name")
		},
	))
}

func newUpdateDeploymentConfigTest(gqlClient *client.Client, input graphql.UpdateDeploymentConfigInput, checks ...func(t *testing.T, id int, err error)) func(t *testing.T) {
	return func(t *testing.T) {
		// Define the mutatation for testing, taking the input as a variable
		mut := `mutation UpdateDeploymentConfig($input: UpdateDeploymentConfigInput!) { updateDeploymentConfig(config:$input) { id } }`

		// Make our request to the GraphQL API
		var resp struct {
			UpdateDeploymentConfig struct{ ID string }
		}
		err := gqlClient.Post(mut, &resp, client.Var("input", input))

		// Run checks with error (if any) and resulting DeploymentConfig ID
		for _, check := range checks {
			check(t, convertID(resp.UpdateDeploymentConfig.ID), err)
		}
	}
}

// TestDeleteDeploymentConfig ensures the deleteDeploymentConfig mutation exhibits expected behavior.
func TestDeleteDeploymentConfig(t *testing.T) {
	// Initialize Test Context
	ctx := context.Background()

	// Initialize DB Backend
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	// Initialize sample data
	existingConfig := graph.DeploymentConfig.Create().
		SetName("delete-me").
		SaveX(ctx)

	// Create a new GraphQL client
	gqlClient := client.New(handler.NewDefaultServer(graphql.NewSchema(graph)))

	// Run tests
	t.Run("Existing", newDeleteDeploymentConfigTest(
		gqlClient,
		existingConfig.ID,
		func(t *testing.T, id int, err error) {
			require.NoError(t, err)
			assert.NotZero(t, id)
			assert.Equal(t, existingConfig.ID, id)

			exists := graph.DeploymentConfig.Query().
				Where(deploymentconfig.ID(id)).
				ExistX(ctx)
			assert.False(t, exists)
		},
	))
	t.Run("NotExists", newDeleteDeploymentConfigTest(
		gqlClient,
		-1,
		func(t *testing.T, id int, err error) {
			assert.Zero(t, id)
			assert.ErrorContains(t, err, "ent: deployment_config not found")
		},
	))
}

func newDeleteDeploymentConfigTest(gqlClient *client.Client, id int, checks ...func(t *testing.T, id int, err error)) func(t *testing.T) {
	return func(t *testing.T) {
		// Define the mutatation for testing, taking the input as a variable
		mut := `mutation DeleteDeploymentConfig($input: ID!) { deleteDeploymentConfig(id:$input) }`

		// Make our request to the GraphQL API
		var resp struct {
			DeleteDeploymentConfig string
		}
		err := gqlClient.Post(mut, &resp, client.Var("input", id))

		// Run checks with error (if any) and resulting DeploymentConfig ID
		for _, check := range checks {
			check(t, convertID(resp.DeleteDeploymentConfig), err)
		}
	}
}
