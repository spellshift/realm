package builder

import (
	"context"
	"fmt"
	mathrand "math/rand"
	"time"

	"github.com/google/uuid"
	"realm.pub/tavern/internal/builder/builderpb"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
	"realm.pub/tavern/internal/ent/asset"
	entbuilder "realm.pub/tavern/internal/ent/builder"
	"realm.pub/tavern/internal/ent/buildprofile"
	"realm.pub/tavern/internal/logging"
)

// CreateBuildTaskInput contains the task-level overrides accepted by the build service.
type CreateBuildTaskInput struct {
	ProfileID       int
	TargetOS        c2pb.Host_Platform
	TargetFormat    *builderpb.TargetFormat
	BuildImage      *string
	Transports      []builderpb.BuildProfileTransport
	Tomes           []builderpb.BuildProfileTome
	ArtifactPath    *string
	SetupScript     *string
	PreBuildScript  *string
	PostBuildScript *string
	Unique          *string
}

// CreateDefaultBuildTask creates a task from the seeded Default profile for an
// explicitly chosen builder. It does not require GraphQL authentication or
// select a random healthy builder.
func CreateDefaultBuildTask(ctx context.Context, graph *ent.Client, builderID int, targetOS c2pb.Host_Platform) (*ent.BuildTask, error) {
	profile, err := graph.BuildProfile.Query().Where(buildprofile.Name(DefaultBuildProfileName)).Only(ctx)
	if err != nil {
		return nil, fmt.Errorf("load default build profile: %w", err)
	}
	return CreateBuildTask(ctx, graph, CreateBuildTaskInput{ProfileID: profile.ID, TargetOS: targetOS}, &builderID)
}

// CreateBuildTask records a build, its immutable inputs, and its pending artifact
// in one transaction. A nil builderID uses the manual builder selection policy.
func CreateBuildTask(ctx context.Context, client *ent.Client, input CreateBuildTaskInput, builderID *int) (*ent.BuildTask, error) {
	tx, err := client.Tx(ctx)
	if err != nil {
		return nil, fmt.Errorf("begin build task transaction: %w", err)
	}
	defer tx.Rollback()
	graph := tx.Client()

	profile, err := graph.BuildProfile.Get(ctx, input.ProfileID)
	if err != nil {
		return nil, fmt.Errorf("failed to load build profile: %w", err)
	}

	targetFormat := DefaultTargetFormat
	if input.TargetFormat != nil {
		targetFormat = *input.TargetFormat
	}
	transports := DefaultTransports
	if len(profile.Transports) > 0 {
		transports = profile.Transports
	}
	if len(input.Transports) > 0 {
		transports = input.Transports
	}
	preBuildScript := profile.Prebuildscript
	if input.PreBuildScript != nil {
		preBuildScript = *input.PreBuildScript
	}
	postBuildScript := profile.Postbuildscript
	if input.PostBuildScript != nil {
		postBuildScript = *input.PostBuildScript
	}
	setupScript := profile.Setupscript
	if input.SetupScript != nil {
		setupScript = *input.SetupScript
	}
	buildImage := profile.BuildImage
	if input.BuildImage != nil && *input.BuildImage != "" {
		buildImage = *input.BuildImage
	}
	var unique *string
	if profile.Unique != "" {
		unique = &profile.Unique
	}
	if input.Unique != nil {
		unique = input.Unique
	}
	tomes := profile.Tomes
	if input.Tomes != nil {
		tomes = input.Tomes
	}

	var selected *ent.Builder
	if builderID != nil {
		selected, err = graph.Builder.Get(ctx, *builderID)
		if err != nil {
			return nil, fmt.Errorf("load selected builder: %w", err)
		}
		supported := false
		for _, target := range selected.SupportedTargets {
			if target == input.TargetOS {
				supported = true
				break
			}
		}
		if !supported {
			return nil, fmt.Errorf("builder %d does not support target %s", selected.ID, input.TargetOS.String())
		}
	} else {
		selected, err = selectBuildTaskBuilder(ctx, graph, input.TargetOS)
		if err != nil {
			return nil, err
		}
	}

	setupScript, buildScript, artifactPath, err := ResolveBuildRecipe(profile, setupScript, input.TargetOS, targetFormat)
	if err != nil {
		return nil, fmt.Errorf("resolve build recipe: %w", err)
	}
	if input.ArtifactPath != nil {
		artifactPath = *input.ArtifactPath
	}
	snapshot := NewProfileSnapshot(profile)
	snapshot.BuildImage = buildImage
	snapshot.Setupscript = setupScript
	snapshot.Prebuildscript = preBuildScript
	snapshot.BuildScript = buildScript
	snapshot.Postbuildscript = postBuildScript
	snapshot.ArtifactPath = artifactPath
	snapshot.Transports = transports
	if unique != nil {
		snapshot.Unique = *unique
	}
	bundleContent, err := SnapshotProfile(ctx, graph, snapshot, tomes)
	if err != nil {
		return nil, fmt.Errorf("capture build profile: %w", err)
	}
	var bundleID *int
	if len(bundleContent) > 0 {
		bundle, err := graph.Asset.Create().SetName("BuildInputs-" + uuid.NewString()).SetContent(bundleContent).Save(ctx)
		if err != nil {
			return nil, fmt.Errorf("save build input bundle: %w", err)
		}
		bundleID = &bundle.ID
	}
	var artifact *ent.Asset
	if artifactPath != "" {
		artifact, err = graph.Asset.Create().
			SetName(ArtifactAssetName(input.TargetOS, targetFormat, snapshot.Name)).
			SetContent([]byte{}).
			SetStatus(asset.StatusIN_PROGRESS).
			Save(ctx)
		if err != nil {
			return nil, fmt.Errorf("create build artifact placeholder: %w", err)
		}
	}
	create := graph.BuildTask.Create().
		SetTargetOs(input.TargetOS).
		SetTargetFormat(targetFormat).
		SetArtifactPath(artifactPath).
		SetBuilder(selected).
		SetBuildScript(buildScript).
		SetProfile(profile).
		SetProfileAtCreation(snapshot).
		SetNillableBundleID(bundleID).
		SetSetupscript(setupScript)
	if artifact != nil {
		create.SetArtifact(artifact)
	}
	if unique != nil {
		create.SetUnique(*unique)
	}
	bt, err := create.Save(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to create build task: %w", err)
	}
	if err := tx.Commit(); err != nil {
		return nil, fmt.Errorf("commit build task: %w", err)
	}
	logging.Audit(ctx, "builder", "build_task.created", "build task created",
		"build_task_id", bt.ID,
		"profile_name", profile.Name,
		"target_os", input.TargetOS.String(),
		"target_format", targetFormat.String(),
		"builder_id", selected.ID,
	)
	return bt.Unwrap(), nil
}

func selectBuildTaskBuilder(ctx context.Context, graph *ent.Client, targetOS c2pb.Host_Platform) (*ent.Builder, error) {
	now := time.Now()
	healthyBuilders, err := graph.Builder.Query().
		Where(entbuilder.LastSeenAtNotNil(), entbuilder.LastSeenAtGTE(now.Add(-MaxStaleAge))).All(ctx)
	if err != nil {
		return nil, fmt.Errorf("failed to query builders: %w", err)
	}
	var candidates []*ent.Builder
	for _, b := range healthyBuilders {
		if !BuilderHealthy(*b.LastSeenAt, b.PollInterval, now) {
			continue
		}
		for _, target := range b.SupportedTargets {
			if target == targetOS {
				candidates = append(candidates, b)
				break
			}
		}
	}
	if len(candidates) == 0 {
		return nil, fmt.Errorf("no builder available that supports target %s (or all matching builders are offline)", targetOS.String())
	}
	return candidates[mathrand.Intn(len(candidates))], nil
}
