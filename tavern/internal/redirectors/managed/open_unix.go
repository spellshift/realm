//go:build unix

package managed

import (
	"fmt"
	"os"
	"syscall"
)

func openIdentityFile(path string) (*os.File, error) {
	return os.OpenFile(path, os.O_RDONLY|syscall.O_NOFOLLOW, 0)
}

func verifyFileOwnership(info os.FileInfo) error {
	stat, ok := info.Sys().(*syscall.Stat_t)
	if !ok {
		return fmt.Errorf("unable to verify redirector identity file ownership")
	}
	if stat.Uid != uint32(os.Geteuid()) {
		return fmt.Errorf("redirector identity file is not owned by the current user")
	}
	return nil
}
