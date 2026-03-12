package graphql_test

import (
	"context"
	"testing"
	"time"

	"github.com/99designs/gqlgen/client"
	"github.com/99designs/gqlgen/graphql/handler"
	_ "github.com/mattn/go-sqlite3"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent/enttest"
	"realm.pub/tavern/internal/graphql"
	tavernhttp "realm.pub/tavern/internal/http"
	"realm.pub/tavern/tomes"
)

func TestCreateBuildTask(t *testing.T) {
	ctx := context.Background()
	graph := enttest.Open(t, "sqlite3", "file:ent?mode=memory&cache=shared&_fk=1")
	defer graph.Close()

	git := tomes.NewGitImporter(graph)
	srv := tavernhttp.NewServer(
		tavernhttp.RouteMap{
			"/graphql": handler.NewDefaultServer(graphql.NewSchema(graph, git)),
		},
		tavernhttp.WithAuthenticationBypass(graph),
	)
	gqlClient := client.New(srv, client.Path("/graphql"))

	mutFull := `mutation createBuildTask($input: CreateBuildTaskInput!) {
		createBuildTask(input: $input) {
			id
			targetOs
			targetFormat
			buildImage
			buildScript
			artifactPath
			builder { id }
		}
	}`
	mutIDOnly := `mutation createBuildTask($input: CreateBuildTaskInput!) {
		createBuildTask(input: $input) {
			id
		}
	}`

	t.Run("NoBuilders", func(t *testing.T) {
		graph.BuildProfile.Delete().ExecX(ctx)

		var resp struct {
			CreateBuildTask struct {
				ID string
			}
		}

		profile := graph.BuildProfile.Create().
			SetName("test").
			SaveX(ctx)

		err := gqlClient.Post(mutIDOnly, &resp, client.Var("input", map[string]any{
			"targetOS": "PLATFORM_LINUX",
			"builderProfileID": profile.ID,

		}))
		require.Error(t, err)
		assert.Contains(t, err.Error(), "no builder available")
	})

	t.Run("NoMatchingBuilder", func(t *testing.T) {
		graph.BuildProfile.Delete().ExecX(ctx)

		profile := graph.BuildProfile.Create().
			SetName("test").
			SaveX(ctx)

		// Create a builder that only supports Windows
		graph.Builder.Create().
			SetSupportedTargets([]c2pb.Host_Platform{c2pb.Host_PLATFORM_WINDOWS}).
			SetUpstream("https://example.com").
			SetLastSeenAt(time.Now()).
			SaveX(ctx)

		var resp struct {
			CreateBuildTask struct {
				ID string
			}
		}
		err := gqlClient.Post(mutIDOnly, &resp, client.Var("input", map[string]any{
			"targetOS": "PLATFORM_LINUX",
			"builderProfileID": profile.ID,
		}))
		require.Error(t, err)
		assert.Contains(t, err.Error(), "no builder available")
	})

	t.Run("SingleMatchingBuilder", func(t *testing.T) {
		// Clean up previous builders
		graph.BuildTask.Delete().ExecX(ctx)
		graph.Builder.Delete().ExecX(ctx)
		graph.BuildProfile.Delete().ExecX(ctx)

		linuxBuilder := graph.Builder.Create().
			SetSupportedTargets([]c2pb.Host_Platform{c2pb.Host_PLATFORM_LINUX}).
			SetUpstream("https://example.com").
			SetLastSeenAt(time.Now()).
			SaveX(ctx)

		// Create a builder profile with transports
		var profileResp struct {
			CreateBuilderProfile struct {
				ID string
			}
		}
		err := gqlClient.Post(`mutation createBuilderProfile($input: CreateBuilderProfileInput!, $transports: [BuildTaskTransportInput!]) {
			createBuilderProfile(input: $input, transports: $transports) {
				id
			}
		}`, &profileResp,
			client.Var("input", map[string]any{
				"name": "test-profile",
			}),
			client.Var("transports", []map[string]any{
				{
					"uri":      "https://callback.example.com",
					"interval": 10,
					"type":     "TRANSPORT_GRPC",
				},
			}),
		)
		require.NoError(t, err)

		var resp struct {
			CreateBuildTask struct {
				ID           string
				TargetOs     string
				TargetFormat string
				BuildImage   string
				BuildScript  string
				ArtifactPath string
				Builder      struct {
					ID string
				}
			}
		}
		err = gqlClient.Post(mutFull, &resp, client.Var("input", map[string]any{
			"targetOS":         "PLATFORM_LINUX",
			"builderProfileID": profileResp.CreateBuilderProfile.ID,
		}))
		require.NoError(t, err)
		require.NotEmpty(t, resp.CreateBuildTask.ID)
		assert.Equal(t, "PLATFORM_LINUX", resp.CreateBuildTask.TargetOs)
		assert.Equal(t, "TARGET_FORMAT_BIN", resp.CreateBuildTask.TargetFormat)
		assert.Equal(t, "spellshift/devcontainer:main", resp.CreateBuildTask.BuildImage)
		assert.Contains(t, resp.CreateBuildTask.BuildScript, "cargo build")
		assert.Contains(t, resp.CreateBuildTask.ArtifactPath, "x86_64-unknown-linux-musl")

		// Verify the builder edge
		bt := graph.BuildTask.GetX(ctx, convertID(resp.CreateBuildTask.ID))
		assignedBuilder := bt.QueryBuilder().OnlyX(ctx)
		assert.Equal(t, linuxBuilder.ID, assignedBuilder.ID)
	})

	t.Run("DefaultsApplied", func(t *testing.T) {
		// Clean up
		graph.BuildTask.Delete().ExecX(ctx)
		graph.Builder.Delete().ExecX(ctx)
		graph.BuildProfile.Delete().ExecX(ctx)

		profile := graph.BuildProfile.Create().
			SetName("test").
			SaveX(ctx)


		graph.Builder.Create().
			SetSupportedTargets([]c2pb.Host_Platform{c2pb.Host_PLATFORM_LINUX}).
			SetUpstream("https://example.com").
			SetLastSeenAt(time.Now()).
			SaveX(ctx)

		var resp struct {
			CreateBuildTask struct {
				ID           string
				TargetFormat string
				BuildImage   string
				ArtifactPath string
			}
		}
		// Only specify targetOS; all other fields should get defaults.
		err := gqlClient.Post(`mutation createBuildTask($input: CreateBuildTaskInput!) {
			createBuildTask(input: $input) {
				id
				targetFormat
				buildImage
				artifactPath
			}
		}`, &resp, client.Var("input", map[string]any{
			"targetOS":         "PLATFORM_LINUX",
			"builderProfileID": profile.ID,
		}))
		require.NoError(t, err)
		assert.Equal(t, "TARGET_FORMAT_BIN", resp.CreateBuildTask.TargetFormat)
		assert.Equal(t, "spellshift/devcontainer:main", resp.CreateBuildTask.BuildImage)
		assert.Contains(t, resp.CreateBuildTask.ArtifactPath, "x86_64-unknown-linux-musl")
	})

	t.Run("CustomArtifactPath", func(t *testing.T) {
		// Clean up
		graph.BuildTask.Delete().ExecX(ctx)
		graph.Builder.Delete().ExecX(ctx)
		graph.BuildProfile.Delete().ExecX(ctx)

		profile := graph.BuildProfile.Create().
			SetName("test").
			SaveX(ctx)


		graph.Builder.Create().
			SetSupportedTargets([]c2pb.Host_Platform{c2pb.Host_PLATFORM_LINUX}).
			SetUpstream("https://example.com").
			SetLastSeenAt(time.Now()).
			SaveX(ctx)

		var resp struct {
			CreateBuildTask struct {
				ID           string
				ArtifactPath string
			}
		}
		err := gqlClient.Post(`mutation createBuildTask($input: CreateBuildTaskInput!) {
			createBuildTask(input: $input) {
				id
				artifactPath
			}
		}`, &resp, client.Var("input", map[string]any{
			"targetOS":     "PLATFORM_LINUX",
			"artifactPath": "/custom/path/to/binary",
			"builderProfileID": profile.ID,
		}))
		require.NoError(t, err)
		assert.Equal(t, "/custom/path/to/binary", resp.CreateBuildTask.ArtifactPath)
	})

	t.Run("MultipleMatchingBuilders", func(t *testing.T) {
		// Clean up
		graph.BuildTask.Delete().ExecX(ctx)
		graph.Builder.Delete().ExecX(ctx)
		graph.BuildProfile.Delete().ExecX(ctx)

		profile := graph.BuildProfile.Create().
			SetName("test").
			SaveX(ctx)

		// Create 3 builders that all support Linux
		builders := make(map[int]bool)
		for i := 0; i < 3; i++ {
			b := graph.Builder.Create().
				SetSupportedTargets([]c2pb.Host_Platform{c2pb.Host_PLATFORM_LINUX}).
				SetUpstream("https://example.com").
				SetLastSeenAt(time.Now()).
				SaveX(ctx)
			builders[b.ID] = true
		}

		// Create 20 build tasks and track which builders get selected
		selectedBuilders := make(map[int]bool)
		for i := 0; i < 20; i++ {
			var resp struct {
				CreateBuildTask struct {
					ID string
				}
			}
			err := gqlClient.Post(mutIDOnly, &resp, client.Var("input", map[string]any{
				"targetOS": "PLATFORM_LINUX",
				"builderProfileID": profile.ID,
			}))
			require.NoError(t, err)

			bt := graph.BuildTask.GetX(ctx, convertID(resp.CreateBuildTask.ID))
			assignedBuilder := bt.QueryBuilder().OnlyX(ctx)
			assert.True(t, builders[assignedBuilder.ID], "selected builder should be one of the candidates")
			selectedBuilders[assignedBuilder.ID] = true
		}

		// With 3 builders and 20 attempts, we should see at least 2 different builders selected
		assert.Greater(t, len(selectedBuilders), 1, "expected random selection across multiple builders")
	})

	t.Run("InvalidTargetFormat", func(t *testing.T) {
		// Clean up
		graph.BuildTask.Delete().ExecX(ctx)
		graph.Builder.Delete().ExecX(ctx)
		graph.BuildProfile.Delete().ExecX(ctx)

		profile := graph.BuildProfile.Create().
			SetName("test").
			SaveX(ctx)

		graph.Builder.Create().
			SetSupportedTargets([]c2pb.Host_Platform{c2pb.Host_PLATFORM_LINUX}).
			SetUpstream("https://example.com").
			SetLastSeenAt(time.Now()).
			SaveX(ctx)

		var resp struct {
			CreateBuildTask struct {
				ID string
			}
		}
		// cdylib is not supported on Linux
		err := gqlClient.Post(mutIDOnly, &resp, client.Var("input", map[string]any{
			"targetOS":     "PLATFORM_LINUX",
			"targetFormat": "TARGET_FORMAT_CDYLIB",
			"builderProfileID": profile.ID,
		}))
		require.Error(t, err)
		assert.Contains(t, err.Error(), "not supported")
	})

	t.Run("WindowsServiceFormat", func(t *testing.T) {
		// Clean up
		graph.BuildTask.Delete().ExecX(ctx)
		graph.Builder.Delete().ExecX(ctx)
		graph.BuildProfile.Delete().ExecX(ctx)

		profile := graph.BuildProfile.Create().
			SetName("test").
			SetName("test").
			SaveX(ctx)

		graph.Builder.Create().
			SetSupportedTargets([]c2pb.Host_Platform{c2pb.Host_PLATFORM_WINDOWS}).
			SetUpstream("https://example.com").
			SetLastSeenAt(time.Now()).
			SaveX(ctx)

		var resp struct {
			CreateBuildTask struct {
				ID           string
				TargetFormat string
				BuildScript  string
			}
		}
		err := gqlClient.Post(`mutation createBuildTask($input: CreateBuildTaskInput!) {
			createBuildTask(input: $input) {
				id
				targetFormat
				buildScript
			}
		}`, &resp, client.Var("input", map[string]any{
			"targetOS":     "PLATFORM_WINDOWS",
			"targetFormat": "TARGET_FORMAT_WINDOWS_SERVICE",
			"builderProfileID": profile.ID,
		}))
		require.NoError(t, err)
		assert.Equal(t, "TARGET_FORMAT_WINDOWS_SERVICE", resp.CreateBuildTask.TargetFormat)
		assert.Contains(t, resp.CreateBuildTask.BuildScript, "win_service")
	})
}
