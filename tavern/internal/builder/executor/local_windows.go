//go:build windows

package executor

import (
	"os/exec"
)

func configureProcessGroup(cmd *exec.Cmd) {
	// On Windows, child processes are terminated by default when the context is cancelled.
}
