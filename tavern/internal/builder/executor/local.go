package executor

import (
	"context"
	"errors"
	"fmt"
	"log/slog"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"sync"
)

// LocalExecutor executes build tasks on the local host shell inside a
// temporary scratch workspace.
//
// LocalExecutor is intended for fast, containerless testing only (e.g. the
// builder e2e test suite). Unlike DockerExecutor, builds run unsandboxed as
// the builder process: the build script sees the full host filesystem and
// inherits the builder's environment. It is only supported on Linux, where
// the e2e tests exercise it with `/bin/sh`; it is not a portable substitute
// for DockerExecutor on other build hosts. See the "Local Executor" section
// of the package README for details.
type LocalExecutor struct{}

// NewLocalExecutor creates a new LocalExecutor.
func NewLocalExecutor() *LocalExecutor {
	return &LocalExecutor{}
}

// Build creates a temporary workspace, populates /scripts and /tomes from the
// BuildSpec, and executes the scripts sequentially via a local shell entrypoint.
// Output lines are streamed to outputCh and error lines to errorCh.
// Both channels are closed before Build returns.
func (l *LocalExecutor) Build(ctx context.Context, spec BuildSpec, outputCh chan<- string, errorCh chan<- string) (*BuildResult, error) {
	defer close(outputCh)
	defer close(errorCh)

	// LocalExecutor shells out via `/bin/sh`, which is only guaranteed to
	// exist on Linux build hosts. It is for testing only (see type doc), so
	// rather than attempt to emulate a POSIX shell on Windows/macOS, fail
	// fast with a clear error instead of silently misbehaving.
	if runtime.GOOS != "linux" {
		return nil, fmt.Errorf("local executor is only supported on linux (for testing), got GOOS=%q", runtime.GOOS)
	}

	// Prepare local scratch directory with /scripts and /tomes.
	tmpDir, err := prepareMountDir(spec)
	if err != nil {
		if tmpDir != "" {
			os.RemoveAll(tmpDir)
		}
		return nil, fmt.Errorf("failed to prepare workspace dir: %w", err)
	}
	defer os.RemoveAll(tmpDir)

	slog.InfoContext(ctx, "starting local build", "task_id", spec.TaskID, "workspace", tmpDir)

	// Execute all scripts in scripts/ in alphabetical order, mirroring Docker's entrypoint.
	entrypoint := strings.Join([]string{
		"set -e",
		"for s in $(ls scripts/*.sh 2>/dev/null | sort); do echo \"==> Running $s\"; sh \"$s\"; done",
	}, " && ")

	cmd := exec.CommandContext(ctx, "/bin/sh", "-c", entrypoint)
	cmd.Dir = tmpDir

	// Inherit host environment augmented with spec.Env and workspace paths.
	env := append(os.Environ(), spec.Env...)
	env = append(env,
		fmt.Sprintf("REALM_WORKSPACE_DIR=%s", tmpDir),
		fmt.Sprintf("REALM_TOMES_DIR=%s", filepath.Join(tmpDir, "tomes")),
	)
	cmd.Env = env

	configureProcessGroup(cmd)

	stdoutPipe, err := cmd.StdoutPipe()
	if err != nil {
		return nil, fmt.Errorf("failed to create stdout pipe: %w", err)
	}
	stderrPipe, err := cmd.StderrPipe()
	if err != nil {
		return nil, fmt.Errorf("failed to create stderr pipe: %w", err)
	}

	if err := cmd.Start(); err != nil {
		return nil, fmt.Errorf("failed to start build command: %w", err)
	}

	var wg sync.WaitGroup
	wg.Add(2)

	go func() {
		defer wg.Done()
		streamBuildLines(ctx, stdoutPipe, outputCh)
	}()

	go func() {
		defer wg.Done()
		streamBuildLines(ctx, stderrPipe, errorCh)
	}()

	wg.Wait()

	waitErr := cmd.Wait()
	var exitCode int64
	if waitErr != nil {
		var exitErr *exec.ExitError
		if errors.As(waitErr, &exitErr) {
			exitCode = int64(exitErr.ExitCode())
		} else {
			return nil, fmt.Errorf("command execution error: %w", waitErr)
		}
	}

	buildResult := BuildResult{ExitCode: exitCode}
	if exitCode != ExpectedExitCode {
		return &buildResult, fmt.Errorf("command exited with status %d", exitCode)
	}

	if spec.ArtifactPath == "" {
		return &buildResult, nil
	}

	data, name, extractErr := extractLocalArtifact(tmpDir, spec.ArtifactPath)
	if extractErr != nil {
		// Build already reported ExpectedExitCode, so without a distinct
		// error the task would otherwise show as a successful build with no
		// artifact. Surface it through the existing build-error stream
		// (client.go sets StreamBuildTaskOutputRequest.Error from this
		// return value) so it's indistinguishable from any other failure.
		slog.WarnContext(ctx, "artifact extraction failed",
			"task_id", spec.TaskID, "path", spec.ArtifactPath, "error", extractErr)
		return &buildResult, fmt.Errorf("artifact extraction failed: %w", extractErr)
	}

	buildResult.Artifact = data
	buildResult.ArtifactName = name
	slog.InfoContext(ctx, "artifact extracted",
		"task_id", spec.TaskID, "name", name, "size", len(data))

	return &buildResult, nil
}

// extractLocalArtifact reads the artifact file from disk.
// If artifactPath is relative, it is resolved against workspaceDir.
// If artifactPath is absolute and exists on the host, it is read directly.
// If artifactPath is absolute but does not exist on the host, it falls back to
// looking inside workspaceDir stripped of leading separators.
func extractLocalArtifact(workspaceDir, artifactPath string) ([]byte, string, error) {
	resolvedPath := artifactPath
	if !filepath.IsAbs(resolvedPath) {
		resolvedPath = filepath.Join(workspaceDir, resolvedPath)
	} else if _, err := os.Stat(resolvedPath); err != nil {
		trimmed := strings.TrimPrefix(resolvedPath, string(filepath.Separator))
		altPath := filepath.Join(workspaceDir, trimmed)
		if _, altErr := os.Stat(altPath); altErr == nil {
			resolvedPath = altPath
		}
	}

	info, err := os.Stat(resolvedPath)
	if err != nil {
		return nil, "", fmt.Errorf("artifact file not found at %q: %w", resolvedPath, err)
	}
	if info.IsDir() {
		return nil, "", fmt.Errorf("artifact path %q is a directory, expected a file", resolvedPath)
	}

	data, err := os.ReadFile(resolvedPath)
	if err != nil {
		return nil, "", fmt.Errorf("failed to read artifact %q: %w", resolvedPath, err)
	}

	return data, filepath.Base(resolvedPath), nil
}
