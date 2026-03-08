package executor

import (
	"archive/tar"
	"bufio"
	"context"
	"encoding/base64"
	"fmt"
	"io"
	"log/slog"
	"path/filepath"
	"strings"

	"github.com/docker/docker/api/types/container"
	"github.com/docker/docker/api/types/image"
	"github.com/docker/docker/client"
	"github.com/docker/docker/pkg/stdcopy"
)

// DockerExecutor runs build tasks inside Docker containers.
// It pulls the specified image, creates a container using the build script
// as the entrypoint, and streams stdout/stderr over the provided channels.
type DockerExecutor struct {
	client client.APIClient
}

// NewDockerExecutor creates a DockerExecutor using the provided Docker API client.
func NewDockerExecutor(cli client.APIClient) *DockerExecutor {
	return &DockerExecutor{client: cli}
}

// NewDockerExecutorFromEnv creates a DockerExecutor using the default
// Docker client configuration from environment variables.
func NewDockerExecutorFromEnv(ctx context.Context) (*DockerExecutor, error) {
	cli, err := client.NewClientWithOpts(client.FromEnv, client.WithAPIVersionNegotiation())
	if err != nil {
		return nil, fmt.Errorf("failed to create docker client: %w", err)
	}
	return &DockerExecutor{client: cli}, nil
}

// Build pulls the build image, starts a container with the build script as
// the shell entrypoint, and streams output/error lines over the channels.
// After the container exits successfully, if spec.ArtifactPath is set, the
// artifact is copied from the stopped container before removal.
func (d *DockerExecutor) Build(ctx context.Context, spec BuildSpec, outputCh chan<- string, errorCh chan<- string) (*BuildResult, error) {
	defer close(outputCh)
	defer close(errorCh)

	slog.InfoContext(ctx, "pulling docker image", "image", spec.BuildImage, "task_id", spec.TaskID)

	pullReader, err := d.client.ImagePull(ctx, spec.BuildImage, image.PullOptions{})
	if err != nil {
		return nil, fmt.Errorf("failed to pull image %q: %w", spec.BuildImage, err)
	}
	// Drain and close the pull output to ensure the image is fully downloaded.
	if _, err := io.Copy(io.Discard, pullReader); err != nil {
		pullReader.Close()
		return nil, fmt.Errorf("error reading image pull output: %w", err)
	}
	pullReader.Close()

	slog.InfoContext(ctx, "creating container", "image", spec.BuildImage, "task_id", spec.TaskID)

	// Execute the pre-build script, build, and post-build script within the container.
	var finalScript string
	finalScript += "set -e\n"

	if spec.PreBuildScript != "" {
		finalScript += "echo 'Running Pre-Build Script'\n"
		finalScript += spec.PreBuildScript + "\n"
	}

	finalScript += "echo 'Running Main Build Script'\n"
	// Replace the final build command with just downloading and moving the files.
	// Wait, `spec.BuildScript` already contains the download and the build command (e.g. `cd ... && git clone ... && cargo build ...`)
	// We want to run `spec.BuildScript` but *intercept* the build to insert tomes into the correct directory.
	// `spec.BuildScript` looks like: `cd /home/vscode && git clone ... && cd realm/implants/imix && cargo build ...`
	// If we just run the build script, the tomes won't be copied.
	// We can instead replace `cargo build` inside `spec.BuildScript` with a setup step, or we can just let `spec.BuildScript` run as is?
	// Wait! `spec.BuildScript` will just clone the code, we need to inject the tomes *before* it gets compiled!
	// It's much easier to just run `spec.BuildScript` as two parts, or modify the string.
	// Or we can just create a `wrapper_script.sh` that clones the repo, modifies it, and then compiles.

	// Let's modify `GenerateBuildScript` to return the individual parts instead of a single string. But since it's already in the DB... we can just string replace `&& cargo` with our injection script `&& <INJECT> && cargo`.

	// Or, actually, since `spec.BuildScript` comes from `BuildTask`, and `BuildTask` is only created *now* with the new schema, let's just modify `GenerateBuildScript` so we can use its output. Wait, `GenerateBuildScript` doesn't include the tomes because they are passed separately. Let's just modify `GenerateBuildScript` to NOT include the `cargo build`, and we add it here?
	// No, the `BuildScript` is useful to see in the UI exactly what ran. We should inject it there!

	// Wait, if the prompt says: "Update the builders build command to first: Delete the default install_scripts and place the selected tomes in it."
	// That implies the builder itself (the client pulling tasks) should do it. Which is here in the executor!
	// If we look at the executor:

	// Let's just create a script that runs everything.
	// But `spec.BuildScript` already has `cd /home/vscode && git clone https://github.com/spellshift/realm.git realm && cd realm/implants/imix && cargo build ...`
	// We can split it by `&& cargo build`!

	parts := strings.Split(spec.BuildScript, " && cargo ")
	if len(parts) == 2 {
		setupCmd := parts[0]
		buildCmd := "cargo " + parts[1]

		finalScript += setupCmd + "\n"

		finalScript += `rm -rf install_scripts/*` + "\n"

		for i, tome := range spec.Tomes {
			tomeDir := fmt.Sprintf("install_scripts/tome_%d", i)
			finalScript += fmt.Sprintf("mkdir -p %s\n", tomeDir)

			// Escape quotes in params and script
			var tomeScript string
			tomeScript += `input_params = {` + "\n"
			for k, v := range tome.Params {
				tomeScript += fmt.Sprintf(`  "%s": "%s",`+"\n", strings.ReplaceAll(k, `"`, `\"`), strings.ReplaceAll(v, `"`, `\"`))
			}
			tomeScript += `}` + "\n"
			tomeScript += tome.Script

			tomeScriptB64 := base64.StdEncoding.EncodeToString([]byte(tomeScript))
			finalScript += fmt.Sprintf("echo '%s' | base64 -d > %s/main.eldritch\n", tomeScriptB64, tomeDir)

			// Copy assets
			for _, asset := range tome.Assets {
				assetB64 := base64.StdEncoding.EncodeToString(asset.Content)
				finalScript += fmt.Sprintf("echo '%s' | base64 -d > %s/%s\n", assetB64, tomeDir, asset.Name)
			}
		}

		finalScript += buildCmd + "\n"
	} else {
		finalScript += spec.BuildScript + "\n"
	}

	if spec.PostBuildScript != "" {
		finalScript += "echo 'Running Post-Build Script'\n"
		finalScript += spec.PostBuildScript + "\n"
	}

	resp, err := d.client.ContainerCreate(ctx,
		&container.Config{
			Image:      spec.BuildImage,
			Entrypoint: []string{"/bin/sh", "-c", finalScript},
			Env:        spec.Env,
		},
		nil, // host config
		nil, // networking config
		nil, // platform
		"",  // container name (auto-generated)
	)
	if err != nil {
		return nil, fmt.Errorf("failed to create container: %w", err)
	}
	containerID := resp.ID

	// Ensure the container is removed when we're done.
	defer func() {
		removeErr := d.client.ContainerRemove(context.Background(), containerID, container.RemoveOptions{Force: true})
		if removeErr != nil {
			slog.Warn("failed to remove container", "container_id", containerID, "error", removeErr)
		}
	}()

	if err := d.client.ContainerStart(ctx, containerID, container.StartOptions{}); err != nil {
		return nil, fmt.Errorf("failed to start container: %w", err)
	}

	slog.InfoContext(ctx, "container started", "container_id", containerID, "task_id", spec.TaskID)

	// Attach to container logs to stream stdout and stderr.
	logReader, err := d.client.ContainerLogs(ctx, containerID, container.LogsOptions{
		ShowStdout: true,
		ShowStderr: true,
		Follow:     true,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to attach to container logs: %w", err)
	}
	defer logReader.Close()

	// Docker multiplexes stdout/stderr into a single stream with headers.
	// stdcopy.StdCopy demultiplexes them.
	stdoutPR, stdoutPW := io.Pipe()
	stderrPR, stderrPW := io.Pipe()

	go func() {
		_, err := stdcopy.StdCopy(stdoutPW, stderrPW, logReader)
		stdoutPW.CloseWithError(err)
		stderrPW.CloseWithError(err)
	}()

	// Stream stderr lines over errorCh in a background goroutine.
	done := make(chan struct{})
	go func() {
		defer close(done)
		scanner := bufio.NewScanner(stderrPR)
		for scanner.Scan() {
			errorCh <- scanner.Text()
		}
	}()

	scanner := bufio.NewScanner(stdoutPR)
	for scanner.Scan() {
		outputCh <- scanner.Text()
	}

	// Wait for stderr goroutine to finish.
	<-done

	// Wait for the container to exit and check its status.
	statusCh, errCh := d.client.ContainerWait(ctx, containerID, container.WaitConditionNotRunning)
	var exitCode int64
	select {
	case err := <-errCh:
		if err != nil {
			return nil, fmt.Errorf("error waiting for container: %w", err)
		}
	case result := <-statusCh:
		exitCode = result.StatusCode
	case <-ctx.Done():
		return nil, ctx.Err()
	}

	// Extract artifact from the stopped container (before deferred removal).
	buildResult := BuildResult{ExitCode: exitCode}

	if exitCode != ExpectedExitCode {
		return &buildResult, fmt.Errorf("container exited with status %d", exitCode)
	}

	if spec.ArtifactPath == "" {
		return &buildResult, nil
	}

	data, name, extractErr := d.extractArtifact(ctx, containerID, spec.ArtifactPath)
	if extractErr != nil {
		slog.WarnContext(ctx, "artifact extraction failed",
			"task_id", spec.TaskID, "path", spec.ArtifactPath, "error", extractErr)
		return &buildResult, nil
	}

	buildResult.Artifact = data
	buildResult.ArtifactName = name
	slog.InfoContext(ctx, "artifact extracted",
		"task_id", spec.TaskID, "name", name, "size", len(data))

	return &buildResult, nil
}

// extractArtifact copies a file from a stopped container using the Docker API.
// CopyFromContainer returns a tar archive; this method extracts the first
// regular file from that archive and returns its contents and basename.
func (d *DockerExecutor) extractArtifact(ctx context.Context, containerID, path string) ([]byte, string, error) {
	tarReader, _, err := d.client.CopyFromContainer(ctx, containerID, path)
	if err != nil {
		return nil, "", fmt.Errorf("CopyFromContainer %q: %w", path, err)
	}
	defer tarReader.Close()

	tr := tar.NewReader(tarReader)
	for {
		hdr, err := tr.Next()
		if err == io.EOF {
			break
		}
		if err != nil {
			return nil, "", fmt.Errorf("reading tar entry: %w", err)
		}
		if hdr.Typeflag != tar.TypeReg {
			continue
		}
		data, err := io.ReadAll(tr)
		if err != nil {
			return nil, "", fmt.Errorf("reading artifact data: %w", err)
		}
		return data, filepath.Base(hdr.Name), nil
	}

	return nil, "", fmt.Errorf("no regular file found at %q", path)
}
