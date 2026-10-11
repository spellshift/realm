//go:build windows

// This file exists so the executor package still compiles under GOOS=windows:
// local.go calls configureProcessGroup(cmd) unconditionally in LocalExecutor.Build,
// and Go requires that symbol to exist for every build target, regardless of whether
// it's ever reached at runtime. DockerExecutor has no such restriction and must keep
// working on a Windows-hosted builder, so the package as a whole has to build there.
package executor

import (
	"os/exec"
)

func configureProcessGroup(cmd *exec.Cmd) {
	// On Windows, child processes are terminated by default when the context is cancelled.
	// NOTE: LocalExecutor.Build currently refuses to run on non-Linux hosts (it is
	// testing-only and shells out via `/bin/sh`), so this body is unreachable in
	// practice until that restriction is lifted alongside a portable shell resolution.
}
