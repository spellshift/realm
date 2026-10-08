package builder_test

import (
	"archive/tar"
	"bytes"
	"compress/gzip"
	"context"
	"io"
	"strconv"
	"strings"
	"testing"
	"time"

	"github.com/99designs/gqlgen/client"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"realm.pub/tavern/internal/builder"
	"realm.pub/tavern/internal/builder/builderpb"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
)

// Existing executor fixtures construct tasks directly; capture their profile just
// as the application does, rather than relying on the live-profile fallback.
func snapshotForTest(t *testing.T, graph *ent.Client, profile *ent.BuildProfile) *builderpb.BuildProfileSnapshot {
	t.Helper()
	snapshot := builder.NewProfileSnapshot(profile)
	bundle, err := builder.SnapshotProfile(context.Background(), graph, snapshot, profile.Tomes)
	require.NoError(t, err)
	require.Empty(t, bundle)
	return snapshot
}

func testProfileSnapshotLifecycle(t *testing.T, graph *ent.Client, gql *client.Client, rpc builderpb.BuilderClient, workerID int) {
	ctx := context.Background()
	asset := graph.Asset.Create().SetName("inputs/data.txt").SetContent([]byte("original asset")).SaveX(ctx)
	tome := graph.Tome.Create().SetName("original-tome").SetDescription("snapshot test").
		SetAuthor("test").SetEldritch("print('original')").AddAssets(asset).SaveX(ctx)
	profile := graph.BuildProfile.Create().SetName("Original Profile").SetDescription("snapshot test").
		SetBuildImage("original:image").
		SetSetupscript("echo original setup").SetPrebuildscript("echo original pre").
		SetPostbuildscript("echo original post").
		SetBuildScript("echo {{.TargetTriple}}; {{.BuildCommand}}").
		SetArtifactPath("/original/{{.TargetOS}}/agent").
		SetUnique("{\"original\":true}").
		SetTransports([]builderpb.BuildProfileTransport{{URI: "https://original.example", Interval: 1, Type: c2pb.Transport_TRANSPORT_GRPC, Extra: "original-extra"}}).
		SetTomes([]builderpb.BuildProfileTome{{TomeID: tome.ID, Params: "{\"mode\":\"original\"}"}}).SaveX(ctx)

	create := func() *ent.BuildTask {
		t.Helper()
		var response struct {
			CreateBuildTask struct {
				ID                string
				Bundle            struct{ ID string }
				ProfileAtCreation struct {
					Name, BuildImage, Setupscript, Prebuildscript, BuildScript, Postbuildscript, ArtifactPath, Unique string
					Transports                                                                                        []struct {
						URI, Extra string
						Interval   int
					}
					Tomes []struct {
						TomeID       int
						Name, Params string
					}
				}
			}
		}
		err := gql.Post(`mutation($input: CreateBuildTaskInput!) {
   createBuildTask(input:$input) {
    id bundle { id }
    profileAtCreation {
     name buildImage setupscript prebuildscript buildScript postbuildscript artifactPath unique
     transports { uri interval extra }
     tomes { tomeID name params }
    }
   }
  }`, &response, client.Var("input", map[string]any{"profileID": strconv.Itoa(profile.ID), "targetOS": "PLATFORM_LINUX"}))
		require.NoError(t, err)
		id, err := strconv.Atoi(response.CreateBuildTask.ID)
		require.NoError(t, err)
		task := graph.BuildTask.GetX(ctx, id)
		require.NotNil(t, task.ProfileAtCreation)
		require.NotEmpty(t, response.CreateBuildTask.Bundle.ID)
		assert.Equal(t, task.ProfileAtCreation.Name, response.CreateBuildTask.ProfileAtCreation.Name)
		assert.Equal(t, task.ProfileAtCreation.BuildImage, response.CreateBuildTask.ProfileAtCreation.BuildImage)
		require.Len(t, response.CreateBuildTask.ProfileAtCreation.Transports, 1)
		assert.Equal(t, task.ProfileAtCreation.Transports[0].URI, response.CreateBuildTask.ProfileAtCreation.Transports[0].URI)
		require.Len(t, response.CreateBuildTask.ProfileAtCreation.Tomes, 1)
		assert.Equal(t, task.ProfileAtCreation.Tomes[0].Name, response.CreateBuildTask.ProfileAtCreation.Tomes[0].Name)
		return task
	}

	// Builder liveness is independent of the one-second agent transport interval.
	graph.Builder.UpdateOneID(workerID).SetLastSeenAt(time.Now().Add(-20 * time.Second)).SaveX(ctx)
	original := create()
	originalBundle := original.QueryBundle().OnlyX(ctx)
	assert.Equal(t, profile.ID, original.QueryProfile().OnlyX(ctx).ID)

	// Change both the recipe and every mutable source behind its nested tome data.
	graph.Asset.UpdateOne(asset).SetContent([]byte("updated asset")).SaveX(ctx)
	graph.Tome.UpdateOne(tome).SetName("updated-tome").SetEldritch("print('updated')").SaveX(ctx)
	profile = graph.BuildProfile.UpdateOne(profile).SetName("Updated Profile").
		SetBuildImage("updated:image").SetSetupscript("echo updated setup").
		SetPrebuildscript("echo updated pre").SetPostbuildscript("echo updated post").
		SetBuildScript("echo updated command").SetArtifactPath("/updated/agent").
		SetUnique("{\"updated\":true}").
		SetTransports([]builderpb.BuildProfileTransport{{URI: "https://updated.example", Interval: 300, Type: c2pb.Transport_TRANSPORT_HTTP1}}).
		SetTomes([]builderpb.BuildProfileTome{{TomeID: tome.ID, Params: "{\"mode\":\"updated\"}"}}).SaveX(ctx)
	updated := create()
	assert.Equal(t, "Original Profile", graph.BuildTask.GetX(ctx, original.ID).ProfileAtCreation.Name)
	assert.Equal(t, "Updated Profile", original.QueryProfile().OnlyX(ctx).Name)
	assert.NotEqual(t, originalBundle.ID, updated.QueryBundle().OnlyX(ctx).ID)

	// A failed assignment must roll back the input bundle as well as the task.
	assetsBefore := graph.Asset.Query().CountX(ctx)
	tasksBefore := graph.BuildTask.Query().CountX(ctx)
	graph.Builder.UpdateOneID(workerID).SetLastSeenAt(time.Now().Add(-time.Hour)).SaveX(ctx)
	var failed struct{ CreateBuildTask struct{ ID string } }
	err := gql.Post(`mutation($input: CreateBuildTaskInput!) { createBuildTask(input:$input) { id } }`,
		&failed, client.Var("input", map[string]any{"profileID": strconv.Itoa(profile.ID), "targetOS": "PLATFORM_LINUX"}))
	require.ErrorContains(t, err, "no builder available")
	assert.Equal(t, assetsBefore, graph.Asset.Query().CountX(ctx))
	assert.Equal(t, tasksBefore, graph.BuildTask.Query().CountX(ctx))
	graph.Builder.UpdateOneID(workerID).SetLastSeenAt(time.Now()).SaveX(ctx)

	graph.BuildProfile.UpdateOne(profile).SetBuildScript("{{.UnknownVariable}}").SaveX(ctx)
	err = gql.Post(`mutation($input: CreateBuildTaskInput!) { createBuildTask(input:$input) { id } }`,
		&failed, client.Var("input", map[string]any{"profileID": strconv.Itoa(profile.ID), "targetOS": "PLATFORM_LINUX"}))
	require.ErrorContains(t, err, "resolve build recipe")
	assert.Equal(t, assetsBefore, graph.Asset.Query().CountX(ctx))
	assert.Equal(t, tasksBefore, graph.BuildTask.Query().CountX(ctx))
	graph.BuildProfile.UpdateOne(profile).SetBuildScript("echo updated command").SaveX(ctx)

	// Deleting the source tome must not prevent claiming or downloading either build.
	graph.Tome.DeleteOne(tome).ExecX(ctx)
	missingBefore := graph.Asset.Query().CountX(ctx)
	err = gql.Post(`mutation($input: CreateBuildTaskInput!) { createBuildTask(input:$input) { id } }`,
		&failed, client.Var("input", map[string]any{"profileID": strconv.Itoa(profile.ID), "targetOS": "PLATFORM_LINUX"}))
	require.ErrorContains(t, err, "capture build profile")
	assert.Equal(t, missingBefore, graph.Asset.Query().CountX(ctx))
	assert.Equal(t, tasksBefore, graph.BuildTask.Query().CountX(ctx))

	// Even a tome subsequently added to the live profile is not authorized for old tasks.
	laterTome := graph.Tome.Create().SetName("later-tome").SetDescription("later").
		SetAuthor("test").SetEldritch("print('later')").SaveX(ctx)
	graph.BuildProfile.UpdateOne(profile).
		SetTomes([]builderpb.BuildProfileTome{{TomeID: laterTome.ID}}).SaveX(ctx)

	// Legacy records stay readable and unclaimed; no current profile substitution.
	legacy := graph.BuildTask.Create().SetProfile(profile).SetBuilderID(workerID).
		SetTargetOs(c2pb.Host_PLATFORM_LINUX).SetTargetFormat(builderpb.TargetFormat_TARGET_FORMAT_BIN).
		SetBuildScript("echo legacy").SaveX(ctx)
	claimed, err := rpc.ClaimBuildTasks(ctx, &builderpb.ClaimBuildTasksRequest{})
	require.NoError(t, err)
	require.Len(t, claimed.Tasks, 2)
	specs := make(map[int64]*builderpb.BuildTaskSpec)
	for _, spec := range claimed.Tasks {
		specs[spec.Id] = spec
	}
	oldSpec, newSpec := specs[int64(original.ID)], specs[int64(updated.ID)]
	require.NotNil(t, oldSpec)
	require.NotNil(t, newSpec)
	assert.Equal(t, "original:image", oldSpec.BuildImage)
	assert.Equal(t, "echo original setup", oldSpec.SetupScript)
	assert.Equal(t, "echo original pre", oldSpec.PreBuildScript)
	assert.Equal(t, "echo original post", oldSpec.PostBuildScript)
	assert.Contains(t, oldSpec.BuildScript, "echo x86_64-unknown-linux-musl; cargo build")
	assert.Equal(t, "/original/PLATFORM_LINUX/agent", oldSpec.ArtifactPath)
	assert.Contains(t, strings.Join(oldSpec.Env, "\n"), "https://original.example")
	assert.Contains(t, strings.Join(oldSpec.Env, "\n"), "original-extra")
	assert.Contains(t, oldSpec.Env, "IMIX_UNIQUE={\"original\":true}")
	assert.Equal(t, "original-tome", oldSpec.Tomes[0].Name)
	assert.Equal(t, "{\"mode\":\"original\"}", oldSpec.Tomes[0].Params)
	assert.Equal(t, "updated:image", newSpec.BuildImage)
	assert.Equal(t, "echo updated command", newSpec.BuildScript)
	assert.Equal(t, "/updated/agent", newSpec.ArtifactPath)
	assert.Contains(t, newSpec.Env, "IMIX_UNIQUE={\"updated\":true}")
	assert.Contains(t, strings.Join(newSpec.Env, "\n"), "https://updated.example")
	assert.True(t, graph.BuildTask.GetX(ctx, legacy.ID).ClaimedAt.IsZero())

	for _, tc := range []struct {
		task                  *ent.BuildTask
		name, script, content string
	}{
		{original, "original-tome", "print('original')", "original asset"},
		{updated, "updated-tome", "print('updated')", "updated asset"},
	} {
		stream, err := rpc.DownloadTome(ctx, &builderpb.DownloadTomeRequest{TaskId: int64(tc.task.ID), TomeId: int64(tome.ID)})
		require.NoError(t, err)
		var data bytes.Buffer
		var name string
		for {
			chunk, err := stream.Recv()
			if err == io.EOF {
				break
			}
			require.NoError(t, err)
			if chunk.Name != "" {
				name = chunk.Name
			}
			data.Write(chunk.Chunk)
		}
		assert.Equal(t, tc.name, name)
		gz, err := gzip.NewReader(bytes.NewReader(data.Bytes()))
		require.NoError(t, err)
		archive := tar.NewReader(gz)
		files := make(map[string]string)
		for {
			header, err := archive.Next()
			if err == io.EOF {
				break
			}
			require.NoError(t, err)
			content, err := io.ReadAll(archive)
			require.NoError(t, err)
			files[header.Name] = string(content)
		}
		require.NoError(t, gz.Close())
		assert.Equal(t, tc.script, files["main.eldritch"])
		assert.Equal(t, tc.content, files["inputs/data.txt"])
	}
	denied, err := rpc.DownloadTome(ctx, &builderpb.DownloadTomeRequest{TaskId: int64(original.ID), TomeId: int64(laterTome.ID)})
	require.NoError(t, err)
	_, err = denied.Recv()
	assert.Equal(t, codes.InvalidArgument, status.Code(err))

	// Output naming uses the saved name and upload links the Asset to this task.
	upload, err := rpc.UploadBuildArtifact(ctx)
	require.NoError(t, err)
	require.NoError(t, upload.Send(&builderpb.UploadBuildArtifactRequest{TaskId: int64(original.ID), ArtifactName: "agent", Chunk: []byte("compiled agent")}))
	uploaded, err := upload.CloseAndRecv()
	require.NoError(t, err)
	artifact := original.QueryArtifact().OnlyX(ctx)
	assert.Equal(t, int64(artifact.ID), uploaded.AssetId)
	assert.Contains(t, artifact.Name, "original-profile")
	assert.Equal(t, []byte("compiled agent"), artifact.Content)
	assert.Equal(t, originalBundle.Content, original.QueryBundle().OnlyX(ctx).Content)
}

func testProfileTemplateValidation(t *testing.T, graph *ent.Client) {
	ctx := context.Background()
	_, err := graph.BuildProfile.Create().
		SetName("Invalid Template Profile").
		SetDescription("test").
		SetBuildScript("{{.UnclosedAction").
		Save(ctx)
	require.Error(t, err)

	_, err = graph.BuildProfile.Create().
		SetName("Empty Template Profile").
		SetDescription("test").
		SetBuildScript("   ").
		Save(ctx)
	require.Error(t, err)

	_, err = graph.BuildProfile.Create().
		SetName("Invalid Artifact Template").
		SetDescription("test").
		SetArtifactPath("{{.UnclosedAction").
		Save(ctx)
	require.Error(t, err)
}

func testTaskOverridesApplyToSnapshot(t *testing.T, graph *ent.Client, gql *client.Client, rpc builderpb.BuilderClient, workerID int) {
	ctx := context.Background()
	graph.Builder.UpdateOneID(workerID).SetLastSeenAt(time.Now()).SaveX(ctx)

	profileTome := graph.Tome.Create().SetName("profile-tome").SetDescription("test").
		SetAuthor("test").SetEldritch("print('profile')").SaveX(ctx)
	overrideTome := graph.Tome.Create().SetName("override-tome").SetDescription("test").
		SetAuthor("test").SetEldritch("print('override')").SaveX(ctx)

	baseProfile := graph.BuildProfile.Create().
		SetName("Base Profile").
		SetDescription("test base profile").
		SetBuildImage("base:image").
		SetSetupscript("echo base setup").
		SetPrebuildscript("echo base pre").
		SetPostbuildscript("echo base post").
		SetArtifactPath("/base/agent").
		SetUnique("{\"base\":true}").
		SetTransports([]builderpb.BuildProfileTransport{{URI: "https://base.example", Interval: 10, Type: c2pb.Transport_TRANSPORT_GRPC}}).
		SetTomes([]builderpb.BuildProfileTome{{TomeID: profileTome.ID, Params: "{\"mode\":\"base\"}"}}).
		SaveX(ctx)

	var response struct {
		CreateBuildTask struct {
			ID                string
			ProfileAtCreation struct {
				Name            string
				BuildImage      string
				Setupscript     string
				Prebuildscript  string
				BuildScript     string
				Postbuildscript string
				ArtifactPath    string
				Unique          string
				Transports      []struct {
					URI   string
					Extra string
					Type  string
				}
				Tomes []struct {
					TomeID int
					Name   string
					Params string
				}
			}
		}
	}

	err := gql.Post(`mutation($input: CreateBuildTaskInput!) {
    createBuildTask(input:$input) {
      id
      profileAtCreation {
        name buildImage setupscript prebuildscript buildScript postbuildscript artifactPath unique
        transports { uri extra type }
        tomes { tomeID name params }
      }
    }
  }`, &response, client.Var("input", map[string]any{
		"profileID":       strconv.Itoa(baseProfile.ID),
		"targetOS":        "PLATFORM_LINUX",
		"buildImage":      "override:image",
		"setupScript":     "echo override setup",
		"preBuildScript":  "echo override pre",
		"postBuildScript": "echo override post",
		"artifactPath":    "/override/agent",
		"unique":          "{\"override\":true}",
		"transports": []map[string]any{{
			"uri":      "https://override.example",
			"interval": 42,
			"type":     "TRANSPORT_HTTP1",
			"extra":    "override-extra",
		}},
		"tomes": []map[string]any{{
			"tomeID": overrideTome.ID,
			"params": "{\"mode\":\"override\"}",
		}},
	}))
	require.NoError(t, err)

	snapshot := response.CreateBuildTask.ProfileAtCreation
	assert.Equal(t, "override:image", snapshot.BuildImage)
	assert.Equal(t, "echo override setup", snapshot.Setupscript)
	assert.Equal(t, "echo override pre", snapshot.Prebuildscript)
	assert.Equal(t, "echo override post", snapshot.Postbuildscript)
	assert.Equal(t, "/override/agent", snapshot.ArtifactPath)
	assert.Equal(t, "{\"override\":true}", snapshot.Unique)
	require.Len(t, snapshot.Transports, 1)
	assert.Equal(t, "https://override.example", snapshot.Transports[0].URI)
	assert.Equal(t, "override-extra", snapshot.Transports[0].Extra)
	require.Len(t, snapshot.Tomes, 1)
	assert.Equal(t, overrideTome.ID, snapshot.Tomes[0].TomeID)
	assert.Equal(t, "override-tome", snapshot.Tomes[0].Name)
	assert.Equal(t, "{\"mode\":\"override\"}", snapshot.Tomes[0].Params)

	taskID, err := strconv.Atoi(response.CreateBuildTask.ID)
	require.NoError(t, err)

	claimed, err := rpc.ClaimBuildTasks(ctx, &builderpb.ClaimBuildTasksRequest{})
	require.NoError(t, err)
	var spec *builderpb.BuildTaskSpec
	for _, s := range claimed.Tasks {
		if s.Id == int64(taskID) {
			spec = s
			break
		}
	}
	require.NotNil(t, spec)
	assert.Equal(t, "override:image", spec.BuildImage)
	assert.Equal(t, "echo override setup", spec.SetupScript)
	assert.Equal(t, "echo override pre", spec.PreBuildScript)
	assert.Equal(t, "echo override post", spec.PostBuildScript)
	assert.Equal(t, "/override/agent", spec.ArtifactPath)
	assert.Contains(t, spec.Env, "IMIX_UNIQUE={\"override\":true}")
	assert.Contains(t, strings.Join(spec.Env, "\n"), "https://override.example")
	assert.Contains(t, strings.Join(spec.Env, "\n"), "override-extra")
	require.Len(t, spec.Tomes, 1)
	assert.Equal(t, "override-tome", spec.Tomes[0].Name)
	assert.Equal(t, "{\"mode\":\"override\"}", spec.Tomes[0].Params)

	// Download overridden tome from the task's frozen bundle.
	stream, err := rpc.DownloadTome(ctx, &builderpb.DownloadTomeRequest{TaskId: int64(taskID), TomeId: int64(overrideTome.ID)})
	require.NoError(t, err)
	chunk, err := stream.Recv()
	require.NoError(t, err)
	assert.Equal(t, "override-tome", chunk.Name)
}
