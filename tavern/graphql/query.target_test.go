package graphql_test

import (
	"context"
	"testing"

	"github.com/99designs/gqlgen/client"
	"github.com/99designs/gqlgen/graphql/handler"
	"github.com/kcarretto/realm/tavern/ent"
	"github.com/kcarretto/realm/tavern/ent/enttest"
	"github.com/kcarretto/realm/tavern/graphql"
	"github.com/stretchr/testify/assert"

	_ "github.com/mattn/go-sqlite3"
)

// TestQueryTargets ensures the target root query exhibits expected behavior.
func TestQueryTargets(t *testing.T) {
	// Initialize Test Context
	ctx := context.Background()

	// Initialize DB Backend
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	// Initialize sample data
	type testTarget struct {
		name string
		ip   string
	}
	createTargets := func(testTargets ...testTarget) (targetIDs []int) {
		for _, target := range testTargets {
			targetIDs = append(targetIDs,
				graph.Target.
					Create().
					SetName(target.name).
					SetForwardConnectIP(target.ip).
					SaveX(ctx).ID,
			)
		}
		return
	}
	targetIDs := createTargets(
		testTarget{"G1 - Target1", "10.0.0.1"},
		testTarget{"G2 - Target2", "10.0.0.2"},
		testTarget{"G2 - Target3", "10.1.0.1"},
	)

	// Create a new GraphQL client
	gqlClient := client.New(handler.NewDefaultServer(graphql.NewSchema(graph)))

	// Run Tests
	t.Run("All", newQueryTargetsTest(
		gqlClient,
		nil,
		targetIDs,
	))

	namePrefix := "G2 - "
	t.Run("ByNamePrefix", newQueryTargetsTest(
		gqlClient,
		&ent.TargetWhereInput{
			NameHasPrefix: &namePrefix,
		},
		targetIDs[1:3], // Second and Third
	))

	subnetPrefix := "10.0.0."
	t.Run("BySubnetPrefix", newQueryTargetsTest(
		gqlClient,
		&ent.TargetWhereInput{
			ForwardConnectIPHasPrefix: &subnetPrefix,
		},
		targetIDs[:2], // First and Second
	))

}

func newQueryTargetsTest(gqlClient *client.Client, where *ent.TargetWhereInput, expectedTargetIDs []int) func(t *testing.T) {
	return func(t *testing.T) {
		// Define the mutatation for testing, taking the input as a variable
		query := `query QueryTargets($where: TargetWhereInput) { targets(where:$where) { edges { node { id } } } }`

		// Define what our response looks like (Relay Compliant)
		var resp struct {
			Targets struct {
				Edges []struct{ Node struct{ ID string } }
			}
		}

		// Set request variables based on what is provided
		var vars []client.Option
		if where != nil {
			vars = append(vars, client.Var("where", where))
		}

		// Make our request to the GraphQL API
		gqlClient.MustPost(query, &resp, vars...)

		// Collect resulting target ids
		targetIDs := make([]int, 0, len(resp.Targets.Edges))
		for _, edge := range resp.Targets.Edges {
			targetIDs = append(targetIDs, convertID(edge.Node.ID))
		}

		// Ensure lists of targets are the same
		assert.Len(t, targetIDs, len(resp.Targets.Edges))
		assert.Len(t, resp.Targets.Edges, len(expectedTargetIDs))
		assert.Len(t, targetIDs, len(expectedTargetIDs))
		assert.Equal(t, expectedTargetIDs, targetIDs)
	}
}
