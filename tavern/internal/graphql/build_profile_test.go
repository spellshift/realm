package graphql_test

import (
	"context"
	"strconv"
	"testing"

	"github.com/99designs/gqlgen/client"
	"github.com/99designs/gqlgen/graphql/handler"
	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/builder"
	"realm.pub/tavern/internal/builder/builderpb"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/graphql"
	tavernhttp "realm.pub/tavern/internal/http"
	"realm.pub/tavern/tomes"
)

func TestBuildProfileMutations(t *testing.T) {
	ctx := context.Background()
	graph := enttest.OpenTempDB(t)
	defer graph.Close()

	git := tomes.NewGitImporter(graph)
	srv := tavernhttp.NewServer(
		tavernhttp.RouteMap{
			"/graphql": handler.NewDefaultServer(graphql.NewSchema(graph, git)),
		},
		tavernhttp.WithAuthenticationBypass(graph),
	)
	gqlClient := client.New(srv, client.Path("/graphql"))

	updateMut := `mutation updateBuildProfile($profileID: ID!, $input: UpdateBuildProfileInput!) {
		updateBuildProfile(profileID: $profileID, input: $input) {
			id name description buildImage
		}
	}`
	createMut := `mutation createBuildProfile($input: CreateBuildProfileInput!) {
		createBuildProfile(input: $input) { id name }
	}`
	deleteMut := `mutation deleteBuildProfile($profileID: ID!) {
		deleteBuildProfile(profileID: $profileID)
	}`

	baseInput := func(name string) map[string]any {
		return map[string]any{
			"name":            name,
			"description":     "updated description",
			"prebuildscript":  "echo pre",
			"setupscript":     "echo setup",
			"postbuildscript": "echo post",
			"buildImage":      "updated:image",
		}
	}
	seededDefault := graph.BuildProfile.Create().
		SetName(builder.DefaultBuildProfileName).
		SetDescription("").
		SaveX(ctx)

	t.Run("UpdateBuildProfile", func(t *testing.T) {
		profile := graph.BuildProfile.Create().
			SetName("editable-profile").
			SetDescription("original description").
			SaveX(ctx)

		var resp struct {
			UpdateBuildProfile struct {
				ID, Name, Description, BuildImage string
			}
		}
		err := gqlClient.Post(updateMut, &resp, client.Var("profileID", strconv.Itoa(profile.ID)), client.Var("input", baseInput("renamed-profile")))
		require.NoError(t, err)
		assert.Equal(t, "renamed-profile", resp.UpdateBuildProfile.Name)
		assert.Equal(t, "updated description", resp.UpdateBuildProfile.Description)
		assert.Equal(t, "updated:image", resp.UpdateBuildProfile.BuildImage)
	})

	t.Run("UpdateBuildProfile_DefaultBlocked", func(t *testing.T) {
		var resp struct {
			UpdateBuildProfile struct{ ID string }
		}
		err := gqlClient.Post(updateMut, &resp, client.Var("profileID", strconv.Itoa(seededDefault.ID)), client.Var("input", baseInput(builder.DefaultBuildProfileName)))
		require.Error(t, err)
		assert.Contains(t, err.Error(), "cannot be edited")
	})

	t.Run("CreateBuildProfile_DefaultRejected", func(t *testing.T) {
		var resp struct {
			CreateBuildProfile struct{ ID, Name string }
		}
		err := gqlClient.Post(createMut, &resp, client.Var("input", baseInput(builder.DefaultBuildProfileName)))
		require.Error(t, err)
		assert.Contains(t, err.Error(), "reserved")
	})

	t.Run("UpdateBuildProfile_RenameToDefaultRejected", func(t *testing.T) {
		profile := graph.BuildProfile.Create().
			SetName("another-profile").
			SetDescription("original description").
			SaveX(ctx)
		var resp struct {
			UpdateBuildProfile struct{ ID, Name, Description, BuildImage string }
		}
		err := gqlClient.Post(updateMut, &resp, client.Var("profileID", strconv.Itoa(profile.ID)), client.Var("input", baseInput(builder.DefaultBuildProfileName)))
		require.Error(t, err)
		assert.Contains(t, err.Error(), "reserved")
	})

	t.Run("ExistingDuplicateDefaultRecoverable", func(t *testing.T) {
		duplicate := graph.BuildProfile.Create().
			SetName(builder.DefaultBuildProfileName).
			SetDescription("operator-created duplicate").
			SaveX(ctx)
		var updated struct {
			UpdateBuildProfile struct{ ID, Name, Description, BuildImage string }
		}
		err := gqlClient.Post(updateMut, &updated, client.Var("profileID", strconv.Itoa(duplicate.ID)), client.Var("input", baseInput("recovered-default")))
		require.NoError(t, err)
		assert.Equal(t, "recovered-default", updated.UpdateBuildProfile.Name)

		duplicate = graph.BuildProfile.Create().
			SetName(builder.DefaultBuildProfileName).
			SetDescription("second duplicate").
			SaveX(ctx)
		var deleted struct {
			DeleteBuildProfile string
		}
		err = gqlClient.Post(deleteMut, &deleted, client.Var("profileID", strconv.Itoa(duplicate.ID)))
		require.NoError(t, err)
		assert.Equal(t, strconv.Itoa(duplicate.ID), deleted.DeleteBuildProfile)
	})

	t.Run("DeleteBuildProfile", func(t *testing.T) {
		profile := graph.BuildProfile.Create().
			SetName("deletable-profile").
			SetDescription("will be deleted").
			SaveX(ctx)

		var resp struct {
			DeleteBuildProfile string
		}
		err := gqlClient.Post(deleteMut, &resp, client.Var("profileID", strconv.Itoa(profile.ID)))
		require.NoError(t, err)
		assert.Equal(t, strconv.Itoa(profile.ID), resp.DeleteBuildProfile)

		_, err = graph.BuildProfile.Get(ctx, profile.ID)
		require.Error(t, err)
	})

	t.Run("TransportsRoundTripJitter", func(t *testing.T) {
		profile := graph.BuildProfile.Create().
			SetName("jitter-profile").
			SetDescription("profile with jitter transport").
			SaveX(ctx)

		input := baseInput("jitter-profile")
		input["transports"] = []map[string]any{{
			"uri":      "https://callback.example.com",
			"interval": 10,
			"type":     "TRANSPORT_GRPC",
			"jitter":   0.35,
		}}

		var resp struct {
			UpdateBuildProfile struct {
				ID, Name, Description, BuildImage string
			}
		}
		err := gqlClient.Post(updateMut, &resp, client.Var("profileID", strconv.Itoa(profile.ID)), client.Var("input", input))
		require.NoError(t, err)

		reloaded := graph.BuildProfile.GetX(ctx, profile.ID)
		require.Len(t, reloaded.Transports, 1)
		assert.Equal(t, 0.35, reloaded.Transports[0].Jitter)
	})

	t.Run("DeleteBuildProfile_PreservesBuildTasks", func(t *testing.T) {
		profile := graph.BuildProfile.Create().
			SetName("profile-with-tasks").
			SetDescription("has build tasks referencing it").
			SaveX(ctx)

		dbBuilder := graph.Builder.Create().
			SetIdentifier("builder-cascade-test").
			SetSupportedTargets([]c2pb.Host_Platform{c2pb.Host_PLATFORM_LINUX}).
			SetPollInterval(5).
			SaveX(ctx)

		task := graph.BuildTask.Create().
			SetTargetOs(c2pb.Host_PLATFORM_LINUX).
			SetTargetFormat(builderpb.TargetFormat_TARGET_FORMAT_BIN).
			SetBuildScript("cargo build --release").
			SetProfileID(profile.ID).
			SetProfileAtCreation(&builderpb.BuildProfileSnapshot{Name: profile.Name}).
			SetBuilderID(dbBuilder.ID).
			SaveX(ctx)

		var resp struct {
			DeleteBuildProfile string
		}
		err := gqlClient.Post(deleteMut, &resp, client.Var("profileID", strconv.Itoa(profile.ID)))
		require.NoError(t, err, "deleting a profile with associated build tasks should not fail with a foreign key constraint error")
		assert.Equal(t, strconv.Itoa(profile.ID), resp.DeleteBuildProfile)

		_, err = graph.BuildProfile.Get(ctx, profile.ID)
		require.Error(t, err)

		remaining, err := graph.BuildTask.Get(ctx, task.ID)
		require.NoError(t, err, "build tasks should survive deletion of their profile")
		_, err = remaining.QueryProfile().Only(ctx)
		require.Error(t, err, "deleted profile edge should be cleared")
		require.NotNil(t, remaining.ProfileAtCreation)
		assert.Equal(t, "profile-with-tasks", remaining.ProfileAtCreation.Name)
	})

	t.Run("DeleteBuildProfile_DefaultBlocked", func(t *testing.T) {
		var resp struct {
			DeleteBuildProfile string
		}
		err := gqlClient.Post(deleteMut, &resp, client.Var("profileID", strconv.Itoa(seededDefault.ID)))
		require.Error(t, err)
		assert.Contains(t, err.Error(), "cannot be deleted")

		_, err = graph.BuildProfile.Get(ctx, seededDefault.ID)
		require.NoError(t, err, "default profile should not have been deleted")
	})
}
