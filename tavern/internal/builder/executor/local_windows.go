//go:build windows

package executor

import (
	"os/exec"
)

func configureProcessGroup(cmd *exec.Cmd) {
	// On Windows, child processes are terminated by default when the context is cancelled.
	// NOTE: LocalExecutor.Build currently refuses to run on non-Linux hosts (it is
	// testing-only and shells out via `/bin/sh`), so this is unreachable until that
	// restriction is lifted alongside a portable shell resolution.
}
