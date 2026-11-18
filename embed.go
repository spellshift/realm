package realm

import (
	"embed"
	"io/fs"
	"log/slog"
	"os"
	"path/filepath"
	"strings"
	"sync"
)

//go:embed all:implants
//go:embed all:bin/reflective_loader
//go:embed all:bin/create_file_dll
//go:embed all:bin/embedded_files_test
//go:embed tavern/internal/c2/proto/*.proto
//go:embed tavern/portals/proto/*.proto
//go:embed tavern/internal/www/src/assets/eldritch-docs.json
var AgentSourceFS embed.FS

// warnOnBuildArtifacts logs when the embedded tree contains a non-empty cargo
// target directory. go:embed can't exclude paths, so a binary built from a
// checkout with local Rust build output carries (and extracts) all of it.
func warnOnBuildArtifacts(files []string) {
	for _, f := range files {
		for _, part := range strings.Split(f, "/")[:strings.Count(f, "/")] {
			if part == "target" {
				slog.Warn("embedded agent source contains a non-empty target directory; run `cargo clean` before building tavern to shrink the binary and speed up builds", "example", f)
				return
			}
		}
	}
}

func ExtractAgentSource(dstDir string) error {
	if err := os.MkdirAll(dstDir, 0o755); err != nil {
		return err
	}
	if err := os.Chmod(dstDir, 0o755); err != nil {
		return err
	}
	var files []string
	err := fs.WalkDir(AgentSourceFS, ".", func(path string, d fs.DirEntry, err error) error {
		if err != nil {
			return err
		}
		if path == "." {
			return nil
		}
		targetPath := filepath.Join(dstDir, filepath.FromSlash(path))
		if d.IsDir() {
			if err := os.MkdirAll(targetPath, 0o755); err != nil {
				return err
			}
			return os.Chmod(targetPath, 0o755)
		}
		files = append(files, path)
		return nil
	})
	if err != nil {
		return err
	}

	warnOnBuildArtifacts(files)

	const workers = 16
	ch := make(chan string, len(files))
	for _, f := range files {
		ch <- f
	}
	close(ch)

	var (
		wg       sync.WaitGroup
		firstErr error
		errOnce  sync.Once
	)
	for range workers {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for path := range ch {
				data, readErr := AgentSourceFS.ReadFile(path)
				if readErr != nil {
					errOnce.Do(func() { firstErr = readErr })
					return
				}
				targetPath := filepath.Join(dstDir, filepath.FromSlash(path))
				_ = os.Remove(targetPath)
				f, openErr := os.OpenFile(targetPath, os.O_WRONLY|os.O_CREATE|os.O_EXCL, 0o644)
				if openErr != nil {
					errOnce.Do(func() { firstErr = openErr })
					return
				}
				_, writeErr := f.Write(data)
				closeErr := f.Close()
				if writeErr != nil {
					errOnce.Do(func() { firstErr = writeErr })
					return
				}
				if closeErr != nil {
					errOnce.Do(func() { firstErr = closeErr })
					return
				}
				if chmodErr := os.Chmod(targetPath, 0o644); chmodErr != nil {
					errOnce.Do(func() { firstErr = chmodErr })
					return
				}
			}
		}()
	}
	wg.Wait()
	return firstErr
}
