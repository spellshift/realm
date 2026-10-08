package executor

import (
	"archive/tar"
	"bytes"
	"compress/gzip"
	"fmt"
	"io"
	"os"
	"path/filepath"
)

// prepareMountDir creates a temporary directory with /scripts and /tomes
// subdirectories populated from the BuildSpec. It writes the pre-build,
// build, and post-build scripts to numbered files under /scripts so they
// execute in order. Returns the tmp dir path (caller must clean up).
//
// Shared by DockerExecutor (which mounts/copies the result into a container)
// and LocalExecutor (which runs scripts against it directly on the host).
func prepareMountDir(spec BuildSpec) (string, error) {
	tmpDir, err := os.MkdirTemp("", "realm-build-*")
	if err != nil {
		return "", fmt.Errorf("creating temp dir: %w", err)
	}

	scriptsDir := filepath.Join(tmpDir, "scripts")
	if err := os.MkdirAll(scriptsDir, 0o755); err != nil {
		return tmpDir, fmt.Errorf("creating scripts dir: %w", err)
	}

	tomesDir := filepath.Join(tmpDir, "tomes")
	if err := os.MkdirAll(tomesDir, 0o755); err != nil {
		return tmpDir, fmt.Errorf("creating tomes dir: %w", err)
	}

	// Write setup script.
	if spec.SetupScript != "" {
		if err := os.WriteFile(filepath.Join(scriptsDir, "0_setup.sh"), []byte(spec.SetupScript), 0o755); err != nil {
			return tmpDir, fmt.Errorf("writing setup script: %w", err)
		}
	}

	// Write pre-build script.
	if spec.PreBuildScript != "" {
		if err := os.WriteFile(filepath.Join(scriptsDir, "1_pre_build.sh"), []byte(spec.PreBuildScript), 0o755); err != nil {
			return tmpDir, fmt.Errorf("writing pre-build script: %w", err)
		}
	}

	// Write build script.
	if spec.BuildScript != "" {
		if err := os.WriteFile(filepath.Join(scriptsDir, "4_build.sh"), []byte(spec.BuildScript), 0o755); err != nil {
			return tmpDir, fmt.Errorf("writing build script: %w", err)
		}
	}

	// Write post-build script.
	if spec.PostBuildScript != "" {
		if err := os.WriteFile(filepath.Join(scriptsDir, "9_post_build.sh"), []byte(spec.PostBuildScript), 0o755); err != nil {
			return tmpDir, fmt.Errorf("writing post-build script: %w", err)
		}
	}

	// Copy tomes from source directory if provided.
	if spec.TomesDir != "" {
		err := filepath.Walk(spec.TomesDir, func(path string, info os.FileInfo, err error) error {
			if err != nil {
				return err
			}
			relPath, err := filepath.Rel(spec.TomesDir, path)
			if err != nil {
				return err
			}
			destPath := filepath.Join(tomesDir, relPath)
			if info.IsDir() {
				return os.MkdirAll(destPath, info.Mode())
			}
			data, err := os.ReadFile(path)
			if err != nil {
				return err
			}
			return os.WriteFile(destPath, data, info.Mode())
		})
		if err != nil {
			return tmpDir, fmt.Errorf("copying tomes dir: %w", err)
		}
	}

	// Extract downloaded tome tar.gz archives into per-tome subdirectories.
	for _, t := range spec.Tomes {
		tomeDir := filepath.Join(tomesDir, fmt.Sprintf("%d", t.ID))
		if err := os.MkdirAll(tomeDir, 0o755); err != nil {
			return tmpDir, fmt.Errorf("creating tome dir %d: %w", t.ID, err)
		}

		if err := extractTomeArchive(t.Contents, tomeDir); err != nil {
			return tmpDir, fmt.Errorf("extracting tome %d: %w", t.ID, err)
		}

		// Write params as a JSON file if present.
		if t.Params != "" {
			if err := os.WriteFile(filepath.Join(tomeDir, "params.json"), []byte(t.Params), 0o644); err != nil {
				return tmpDir, fmt.Errorf("writing params for tome %d: %w", t.ID, err)
			}
		}
	}

	return tmpDir, nil
}

// extractTomeArchive decompresses a tar.gz archive and extracts all regular
// files into destDir, preserving their path names and creating subdirectories
// as needed.
func extractTomeArchive(data []byte, destDir string) error {
	gr, err := gzip.NewReader(bytes.NewReader(data))
	if err != nil {
		return fmt.Errorf("opening gzip reader: %w", err)
	}
	defer gr.Close()

	tr := tar.NewReader(gr)
	for {
		hdr, err := tr.Next()
		if err == io.EOF {
			break
		}
		if err != nil {
			return fmt.Errorf("reading tar entry: %w", err)
		}
		if hdr.Typeflag != tar.TypeReg {
			continue
		}

		destPath := filepath.Join(destDir, hdr.Name)

		// Create parent directories for nested asset paths.
		if dir := filepath.Dir(destPath); dir != destDir {
			if err := os.MkdirAll(dir, 0o755); err != nil {
				return fmt.Errorf("creating dir for %s: %w", hdr.Name, err)
			}
		}

		content, err := io.ReadAll(tr)
		if err != nil {
			return fmt.Errorf("reading %s: %w", hdr.Name, err)
		}
		if err := os.WriteFile(destPath, content, 0o644); err != nil {
			return fmt.Errorf("writing %s: %w", hdr.Name, err)
		}
	}

	return nil
}
