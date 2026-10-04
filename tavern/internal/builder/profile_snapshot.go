package builder

import (
	"archive/tar"
	"bytes"
	"context"
	"fmt"
	"io"
	"slices"
	"text/template"

	"realm.pub/tavern/internal/builder/builderpb"
	"realm.pub/tavern/internal/c2/c2pb"
	"realm.pub/tavern/internal/ent"
)

// SnapshotProfile captures profile configuration and packages each tome into a
// tar entry named by its ID. Call inside the task creation transaction.
func SnapshotProfile(ctx context.Context, graph *ent.Client, profile *ent.BuildProfile) (*builderpb.BuildProfileSnapshot, []byte, error) {
	snapshot := &builderpb.BuildProfileSnapshot{
		Name:            profile.Name,
		BuildImage:      profile.BuildImage,
		Setupscript:     profile.Setupscript,
		Prebuildscript:  profile.Prebuildscript,
		BuildScript:     profile.BuildScript,
		Postbuildscript: profile.Postbuildscript,
		ArtifactPath:    profile.ArtifactPath,
		Unique:          profile.Unique,
		Transports:      slices.Clone(profile.Transports),
		Tomes:           make([]builderpb.BuildTomeSnapshot, 0, len(profile.Tomes)),
	}
	if len(profile.Tomes) == 0 {
		return snapshot, nil, nil
	}
	var buf bytes.Buffer
	archive := tar.NewWriter(&buf)
	seen := make(map[int]bool)
	for _, config := range profile.Tomes {
		if seen[config.TomeID] {
			return nil, nil, fmt.Errorf("duplicate tome %d in build profile", config.TomeID)
		}
		seen[config.TomeID] = true
		tome, err := graph.Tome.Get(ctx, config.TomeID)
		if err != nil {
			return nil, nil, fmt.Errorf("snapshot tome %d: %w", config.TomeID, err)
		}
		data, err := PackageTome(ctx, graph, tome.ID)
		if err != nil {
			return nil, nil, err
		}
		if err := archive.WriteHeader(&tar.Header{Name: fmt.Sprintf("%d.tar.gz", tome.ID), Mode: 0600, Size: int64(len(data))}); err != nil {
			return nil, nil, err
		}
		if _, err := archive.Write(data); err != nil {
			return nil, nil, err
		}
		snapshot.Tomes = append(snapshot.Tomes, builderpb.BuildTomeSnapshot{TomeID: tome.ID, Name: tome.Name, Params: config.Params})
	}
	if err := archive.Close(); err != nil {
		return nil, nil, err
	}
	return snapshot, buf.Bytes(), nil
}

// FrozenTome returns a packaged tome from a task's saved input bundle.
func FrozenTome(bundle []byte, tomeID int) ([]byte, error) {
	archive := tar.NewReader(bytes.NewReader(bundle))
	name := fmt.Sprintf("%d.tar.gz", tomeID)
	for {
		header, err := archive.Next()
		if err == io.EOF {
			return nil, fmt.Errorf("tome %d missing from build input bundle", tomeID)
		}
		if err != nil {
			return nil, err
		}
		if header.Name == name {
			return io.ReadAll(archive)
		}
	}
}

// ResolveBuildRecipe renders a profile's command and output path for one target.
func ResolveBuildRecipe(profile *ent.BuildProfile, os c2pb.Host_Platform, format TargetFormat) (string, string, error) {
	if err := ValidateTargetFormat(os, format); err != nil {
		return "", "", err
	}
	command, err := BuildCommand(os, format)
	if err != nil {
		return "", "", err
	}
	values := map[string]string{
		"BuildCommand": command,
		"ArtifactPath": DeriveArtifactPath(os),
		"TargetOS":     os.String(),
		"TargetFormat": format.String(),
		"TargetTriple": buildTarget[os],
	}
	script, err := renderRecipe("build_script", profile.BuildScript, values)
	if err != nil {
		return "", "", err
	}
	path, err := renderRecipe("artifact_path", profile.ArtifactPath, values)
	if err != nil {
		return "", "", err
	}
	return script, path, nil
}

func renderRecipe(name, source string, values map[string]string) (string, error) {
	tmpl, err := template.New(name).Option("missingkey=error").Parse(source)
	if err != nil {
		return "", fmt.Errorf("invalid %s template: %w", name, err)
	}
	var buf bytes.Buffer
	if err := tmpl.Execute(&buf, values); err != nil {
		return "", fmt.Errorf("render %s: %w", name, err)
	}
	if buf.Len() == 0 {
		return "", fmt.Errorf("%s must not be empty", name)
	}
	return buf.String(), nil
}
