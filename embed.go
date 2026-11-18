package realm

import (
	"embed"
	"io/fs"
	"os"
	"path/filepath"
	"sync"
)

//go:embed implants/Cargo.toml
//go:embed implants/rust-toolchain
//go:embed all:implants/.cargo
//go:embed implants/imix/Cargo.toml
//go:embed implants/imix/Cargo.lock
//go:embed implants/imix/build.rs
//go:embed all:implants/imix/src
//go:embed all:implants/imix/install_scripts
//go:embed implants/golem/Cargo.toml
//go:embed implants/golem/build.rs
//go:embed all:implants/golem/src
//go:embed all:implants/golem/tests
//go:embed all:implants/golem/embedded
//go:embed implants/lib/transport/Cargo.toml
//go:embed implants/lib/transport/build.rs
//go:embed all:implants/lib/transport/src
//go:embed implants/lib/pb/Cargo.toml
//go:embed implants/lib/pb/build.rs
//go:embed all:implants/lib/pb/src
//go:embed implants/lib/host_unique/Cargo.toml
//go:embed implants/lib/host_unique/build.rs
//go:embed all:implants/lib/host_unique/src
//go:embed implants/lib/guardrails/Cargo.toml
//go:embed implants/lib/guardrails/build.rs
//go:embed all:implants/lib/guardrails/src
//go:embed implants/lib/netstat/Cargo.toml
//go:embed all:implants/lib/netstat/src
//go:embed implants/lib/eldritch/eldritch-core/Cargo.toml
//go:embed implants/lib/eldritch/eldritch-core/build.rs
//go:embed all:implants/lib/eldritch/eldritch-core/src
//go:embed all:implants/lib/eldritch/eldritch-core/benches
//go:embed all:implants/lib/eldritch/eldritch-core/tests
//go:embed implants/lib/eldritch/eldritch-macros/Cargo.toml
//go:embed all:implants/lib/eldritch/eldritch-macros/src
//go:embed all:implants/lib/eldritch/eldritch-macros/tests
//go:embed implants/lib/eldritch/eldritch-repl/Cargo.toml
//go:embed implants/lib/eldritch/eldritch-repl/build.rs
//go:embed all:implants/lib/eldritch/eldritch-repl/src
//go:embed all:implants/lib/eldritch/eldritch-repl/benches
//go:embed all:implants/lib/eldritch/eldritch-repl/tests
//go:embed implants/lib/eldritch/eldritch-agent/Cargo.toml
//go:embed all:implants/lib/eldritch/eldritch-agent/src
//go:embed implants/lib/eldritch/stdlib/eldritch-libaes/Cargo.toml
//go:embed all:implants/lib/eldritch/stdlib/eldritch-libaes/src
//go:embed implants/lib/eldritch/stdlib/eldritch-libagent/Cargo.toml
//go:embed all:implants/lib/eldritch/stdlib/eldritch-libagent/src
//go:embed implants/lib/eldritch/stdlib/eldritch-libassets/Cargo.toml
//go:embed all:implants/lib/eldritch/stdlib/eldritch-libassets/src
//go:embed implants/lib/eldritch/stdlib/eldritch-libbase64/Cargo.toml
//go:embed all:implants/lib/eldritch/stdlib/eldritch-libbase64/src
//go:embed implants/lib/eldritch/stdlib/eldritch-libcrypto/Cargo.toml
//go:embed all:implants/lib/eldritch/stdlib/eldritch-libcrypto/src
//go:embed implants/lib/eldritch/stdlib/eldritch-libfile/Cargo.toml
//go:embed all:implants/lib/eldritch/stdlib/eldritch-libfile/src
//go:embed implants/lib/eldritch/stdlib/eldritch-libgzip/Cargo.toml
//go:embed all:implants/lib/eldritch/stdlib/eldritch-libgzip/src
//go:embed implants/lib/eldritch/stdlib/eldritch-libhash/Cargo.toml
//go:embed all:implants/lib/eldritch/stdlib/eldritch-libhash/src
//go:embed implants/lib/eldritch/stdlib/eldritch-libhttp/Cargo.toml
//go:embed all:implants/lib/eldritch/stdlib/eldritch-libhttp/src
//go:embed implants/lib/eldritch/stdlib/eldritch-libjson/Cargo.toml
//go:embed all:implants/lib/eldritch/stdlib/eldritch-libjson/src
//go:embed implants/lib/eldritch/stdlib/eldritch-liblinux/Cargo.toml
//go:embed all:implants/lib/eldritch/stdlib/eldritch-liblinux/src
//go:embed implants/lib/eldritch/stdlib/eldritch-libnet/Cargo.toml
//go:embed all:implants/lib/eldritch/stdlib/eldritch-libnet/src
//go:embed implants/lib/eldritch/stdlib/eldritch-libpivot/Cargo.toml
//go:embed implants/lib/eldritch/stdlib/eldritch-libpivot/build.rs
//go:embed all:implants/lib/eldritch/stdlib/eldritch-libpivot/src
//go:embed implants/lib/eldritch/stdlib/eldritch-libportal/Cargo.toml
//go:embed all:implants/lib/eldritch/stdlib/eldritch-libportal/src
//go:embed implants/lib/eldritch/stdlib/eldritch-libprocess/Cargo.toml
//go:embed all:implants/lib/eldritch/stdlib/eldritch-libprocess/src
//go:embed implants/lib/eldritch/stdlib/eldritch-librandom/Cargo.toml
//go:embed all:implants/lib/eldritch/stdlib/eldritch-librandom/src
//go:embed implants/lib/eldritch/stdlib/eldritch-libregex/Cargo.toml
//go:embed all:implants/lib/eldritch/stdlib/eldritch-libregex/src
//go:embed implants/lib/eldritch/stdlib/eldritch-libregistry/Cargo.toml
//go:embed all:implants/lib/eldritch/stdlib/eldritch-libregistry/src
//go:embed implants/lib/eldritch/stdlib/eldritch-libreport/Cargo.toml
//go:embed all:implants/lib/eldritch/stdlib/eldritch-libreport/src
//go:embed implants/lib/eldritch/stdlib/eldritch-libscan/Cargo.toml
//go:embed all:implants/lib/eldritch/stdlib/eldritch-libscan/src
//go:embed implants/lib/eldritch/stdlib/eldritch-libssh/Cargo.toml
//go:embed all:implants/lib/eldritch/stdlib/eldritch-libssh/src
//go:embed implants/lib/eldritch/stdlib/eldritch-libsys/Cargo.toml
//go:embed all:implants/lib/eldritch/stdlib/eldritch-libsys/src
//go:embed implants/lib/eldritch/stdlib/eldritch-libtemplate/Cargo.toml
//go:embed all:implants/lib/eldritch/stdlib/eldritch-libtemplate/src
//go:embed implants/lib/eldritch/stdlib/eldritch-libtime/Cargo.toml
//go:embed all:implants/lib/eldritch/stdlib/eldritch-libtime/src
//go:embed implants/lib/eldritch/stdlib/eldritch-libuser/Cargo.toml
//go:embed all:implants/lib/eldritch/stdlib/eldritch-libuser/src
//go:embed implants/lib/eldritch/stdlib/eldritch-libwindows/Cargo.toml
//go:embed implants/lib/eldritch/stdlib/eldritch-libwindows/build.rs
//go:embed all:implants/lib/eldritch/stdlib/eldritch-libwindows/src
//go:embed implants/lib/eldritch/eldritch/Cargo.toml
//go:embed implants/lib/eldritch/eldritch/build.rs
//go:embed all:implants/lib/eldritch/eldritch/src
//go:embed implants/lib/eldritch/eldritch-lsp/Cargo.toml
//go:embed all:implants/lib/eldritch/eldritch-lsp/src
//go:embed implants/lib/portals/portal-stream/Cargo.toml
//go:embed all:implants/lib/portals/portal-stream/src
//go:embed implants/lib/eldritch/testutils/eldritch-mockagent/Cargo.toml
//go:embed all:implants/lib/eldritch/testutils/eldritch-mockagent/src
//go:embed implants/lib/eldritch/stdlib/eldritch-libchain/Cargo.toml
//go:embed implants/lib/eldritch/stdlib/eldritch-libchain/build.rs
//go:embed all:implants/lib/eldritch/stdlib/eldritch-libchain/src
//go:embed implants/lib/eldritch/stdlib/eldritch-libdns/Cargo.toml
//go:embed all:implants/lib/eldritch/stdlib/eldritch-libdns/src
//go:embed bin/reflective_loader/Cargo.toml
//go:embed bin/reflective_loader/rust-toolchain
//go:embed all:bin/reflective_loader/.cargo
//go:embed all:bin/reflective_loader/src
//go:embed bin/create_file_dll/Cargo.toml
//go:embed bin/create_file_dll/rust-toolchain
//go:embed all:bin/create_file_dll/src
//go:embed all:bin/embedded_files_test
//go:embed tavern/internal/c2/proto/*.proto
//go:embed tavern/portals/proto/*.proto
//go:embed tavern/internal/www/src/assets/eldritch-docs.json
var AgentSourceFS embed.FS

func ExtractAgentSource(dstDir string) error {
	if info, err := os.Lstat(dstDir); err == nil {
		if !info.IsDir() {
			if err := os.Remove(dstDir); err != nil {
				return err
			}
		} else {
			_ = filepath.WalkDir(dstDir, func(path string, d fs.DirEntry, walkErr error) error {
				if walkErr == nil && d.IsDir() {
					_ = os.Chmod(path, 0o755)
				}
				return nil
			})
			entries, err := os.ReadDir(dstDir)
			if err != nil {
				return err
			}
			for _, entry := range entries {
				if err := os.RemoveAll(filepath.Join(dstDir, entry.Name())); err != nil {
					return err
				}
			}
		}
	} else if !os.IsNotExist(err) {
		return err
	}

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
